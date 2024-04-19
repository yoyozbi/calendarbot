use crate::calendar::GCalendar;
use crate::calendar::UpdateCalendarEvent;
use crate::commands;
use crate::types;
use anyhow::{Error, Result};
use google_calendar3::api::Event;
use google_calendar3::chrono::NaiveDate;
use log::{debug, error, info};
use poise::serenity_prelude as serenity;
use std::collections::BTreeMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::Mutex;

async fn on_error(error: poise::FrameworkError<'_, types::Data, Error>) {
    // This is our custom error handler
    // They are many errors that can occur, so we only handle the ones we want to customize
    // and forward the rest to the default handler
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error,)
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

struct LocalCache {
    cache: Arc<serenity::Cache>,
    client: Arc<serenity::Http>,
}
impl LocalCache {
    fn new(client: Arc<serenity::Http>) -> Self {
        Self {
            cache: Arc::new(serenity::Cache::default()),
            client,
        }
    }
}

impl serenity::CacheHttp for LocalCache {
    fn http(&self) -> &serenity::Http {
        &self.client
    }

    fn cache(&self) -> Option<&Arc<serenity::Cache>> {
        Some(&self.cache)
    }
}

#[derive(Debug)]
enum Command {
    SendChannel {
        id: u64,
        message: serenity::CreateMessage,
    },
}

pub struct Discord {
    token: String,
    intents: serenity::GatewayIntents,
    calendar_cache: Arc<Mutex<BTreeMap<String, Vec<Event>>>>,
}

impl Discord {
    pub fn new(token: String, gateway_intents: serenity::GatewayIntents) -> Self {
        Self {
            token,
            intents: gateway_intents,
            calendar_cache: Arc::new(Mutex::new(BTreeMap::new())),
        }
    }

    pub async fn init(
        &mut self,
        calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        data: types::Data,
    ) -> serenity::Client {
        let (worker_tx, worker_rx) = mpsc::channel::<Command>(200);
        let worker_clone = worker_tx.clone();
        let calendar_cache = self.calendar_cache.clone();
        let framework = poise::Framework::builder()
            .options(poise::FrameworkOptions {
                commands: vec![
                    commands::utilities::help(),
                    commands::utilities::uptime(),
                    commands::utilities::age(),
                ],
                on_error: |error| Box::pin(async move { on_error(error).await }),
                pre_command: |ctx| {
                    Box::pin(async move {
                        let channel_name = &ctx
                            .channel_id()
                            .name(&ctx)
                            .await
                            .unwrap_or_else(|_| "<unknown>".to_owned());
                        let author = &ctx.author().name;

                        info!(
                            "{} in {} used slash command '{}'",
                            author,
                            channel_name,
                            &ctx.invoked_command_name()
                        );
                    })
                },
                post_command: |ctx| {
                    Box::pin(async move {
                        debug!(
                            "{} executed command \"{}\"",
                            ctx.author().tag(),
                            ctx.command().qualified_name
                        );
                    })
                },
                ..Default::default()
            })
            .setup(move |ctx, ready, framework| {
                Box::pin(async move {
                    Discord::worker_thread(worker_rx, ctx.http.clone().to_owned());

                    Discord::new_data_thread(worker_clone, calendar_rx, calendar_cache);

                    debug!("Registering commands..");
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                    debug!("Setting activity text...");
                    ctx.set_activity(Some(serenity::ActivityData::listening("/help")));

                    info!("{} is ready !", ready.user.name);

                    Ok(data)
                })
            })
            .build();

        let it = self.intents.clone();
        let client = serenity::Client::builder(self.token.clone(), it)
            .framework(framework)
            .await
            .expect("Failed to create client");

        return client;
    }

    fn new_data_thread(
        worker_tx: mpsc::Sender<Command>,
        mut calendar_rx: mpsc::Receiver<UpdateCalendarEvent>,
        calendar_cache: Arc<Mutex<BTreeMap<String, Vec<Event>>>>,
    ) {
        tokio::spawn(async move {
            while let Some(event) = calendar_rx.recv().await {
                debug!("Received event for calendar {}", event.calendar_id);

                let mut cache = calendar_cache.lock().await;
                let events = cache.entry(event.calendar_id.clone()).or_insert(Vec::new());
                let matching = events
                    .iter()
                    .zip(event.new_events.iter())
                    .filter(|&(a, b)| GCalendar::compare_event(a, b))
                    .count();

                let do_match = matching == event.new_events.len() && matching == events.len();
                debug!(
                    "matching: {} == {} && {} == {}",
                    matching,
                    event.new_events.len(),
                    matching,
                    events.len()
                );

                if do_match && !events.is_empty() {
                    debug!("No new events");
                    continue;
                }

                let embed = Discord::event_to_embed(event.new_events.clone());
                let message = match embed {
                    Ok(embed) => serenity::CreateMessage::new().add_embed(embed),
                    Err(_) => serenity::CreateMessage::new().content("Error creating embed"),
                };

                if !events.is_empty() {
                    events.clear();
                }
                events.extend(event.new_events.clone());

                worker_tx
                    .send(Command::SendChannel {
                        id: 1102198299093647470,
                        message,
                    })
                    .await
                    .unwrap();
            }
        });
    }

    fn worker_thread(mut worker_rx: mpsc::Receiver<Command>, http: Arc<serenity::Http>) {
        tokio::spawn(async move {
            while let Some(cmd) = worker_rx.recv().await {
                let cache = LocalCache::new(http.clone());
                match cmd {
                    Command::SendChannel { id, message } => {
                        debug!("Sending message to channel {}", id);
                        let channel = serenity::ChannelId::new(id);
                        channel.send_message(cache, message).await.unwrap();
                    }
                };
            }
        });
    }

    fn event_to_embed(events: Vec<Event>) -> Result<serenity::CreateEmbed> {
        let mut sorted: BTreeMap<NaiveDate, Vec<Event>> = BTreeMap::new();
        let mut fields: Vec<(String, String, bool)> = vec![];
        for ele in events {
            let start = ele.clone().start.unwrap();
            let date = start.date_time.unwrap();
            sorted
                .entry(date.date_naive())
                .or_insert(Vec::new())
                .push(ele);
        }

        for (date, events) in sorted.iter() {
            let mut field = String::new();
            for event in events {
                field.push_str(&format!(
                    "```{} - {} | {}```\n",
                    event
                        .start
                        .clone()
                        .unwrap()
                        .date_time
                        .unwrap()
                        .format("%H:%m"),
                    event
                        .end
                        .clone()
                        .unwrap()
                        .date_time
                        .unwrap()
                        .format("%H:%m"),
                    event.summary.clone().unwrap()
                ));
            }
            fields.push((date.format("**%A** - %e %B").to_string(), field, false));
        }

        Ok(serenity::CreateEmbed::new().title("Events").fields(fields))
    }
}
