use std::env;
use shuttle_secrets::SecretStore;
use anyhow::{anyhow, Result};

pub struct SecretsUtils {}
impl SecretsUtils {
    pub fn get_secret(name: &str, secret_store: &SecretStore) -> Result<String>{
        match env::var(name) {
            Ok(val) => Ok(val),
            Err(_) => {
               match secret_store.get(name) {
                   Some(val) => Ok(val),
                   None => Err(anyhow!(format!("\"{}\" not found", name)))
               }
            }
        }
    }
}