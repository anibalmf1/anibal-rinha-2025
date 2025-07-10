FROM rust:1.88-alpine AS builder
LABEL authors="anibalmf1"

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    pkgconfig

WORKDIR /usr/src

COPY Cargo.toml ./
COPY src ./src

RUN cargo build --release

FROM alpine:3.21

RUN apk add --no-cache \
      libgcc \
      openssl

WORKDIR /app

COPY --from=builder /usr/src/core/target/release/core .

ENV APP_SERVER_URL=""
ENV APP_SERVER_PORT=""
ENV APP_NATS_URL=""
ENV APP_PAYMENT_PROCESSOR_URL=""
ENV APP_PAYMENT_TOPIC=""

EXPOSE ${APP_SERVER_PORT}

CMD ["./core"]