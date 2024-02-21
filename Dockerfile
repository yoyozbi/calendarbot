
FROM rust:1.76 AS build
RUN USER=root cargo new --bin calendarbot
WORKDIR /app

COPY Cargo.lock Cargo.lock
COPY Cargo.toml Cargo.toml

RUN cargo build --release
RUN rm src/*.rs

COPY src/ src/

RUN rm ./target/release/deps/calendarbot*
RUN cargo build --release


FROM ghcr.io/distroless/cc-debian11
COPY --from=builder /app/target/release/calendarbot /usr/local/bin/calendarbot

CMD ["calendarbot"]