FROM rust:1.88-alpine AS builder
LABEL authors="anibalmf1"

RUN apk add --no-cache \
    musl-dev \
    openssl-dev \
    openssl-libs-static \
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

COPY --from=builder /usr/src/target/release/core .

ENV APP_SERVER_URL=""
ENV APP_SERVER_PORT=""
ENV APP_REDIS_URL=""
ENV APP_PAYMENT_PROCESSOR_URL=""
ENV APP_PAYMENT_TOPIC=""
ENV APP_DB_HOST=""
ENV APP_DB_NAME=""
ENV APP_DB_USER=""
ENV APP_DB_PASSWORD=""

EXPOSE ${APP_SERVER_PORT}

CMD ["./core"]