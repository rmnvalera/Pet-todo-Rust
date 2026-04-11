FROM rust:1.89-slim AS builder
WORKDIR /app
ARG APP_NAME
ENV APP_NAME=$APP_NAME
COPY . .
RUN cargo build --release -p $APP_NAME

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    libssl3 \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /usr/local/bin
ARG APP_NAME
ENV APP_NAME=$APP_NAME
COPY --from=builder /app/target/release/$APP_NAME .
COPY ./Settings.yaml ./Settings.yaml
CMD ./$APP_NAME