# Set the architecture argument (arm64, i.e. aarch64 as default)
# For amd64, i.e. x86_64, you can append a flag when invoking the build `... --build-arg "ARCH=x86_64"`
ARG ARCH=aarch64
ARG BASE_TAG=latest-arm64

FROM rust:1.76 AS build
WORKDIR /app

RUN apt-get update -y && apt-get install -y libpq5
RUN touch .env

COPY . .
RUN cargo build --release 


FROM debian:trixie-slim
WORKDIR /app

RUN apt-get update -y && apt-get install -y libpq5

RUN useradd -u 1000 runner
USER runner


COPY --from=build /app/target/release/calendarbot .

COPY --from=build /app/.env ./.env

ENTRYPOINT ["/app/calendarbot"]

