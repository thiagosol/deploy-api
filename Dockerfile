# 1️⃣ Build Stage
FROM rust:1.84 AS builder

WORKDIR /app

RUN apt update && apt install -y musl-tools

COPY . .

RUN rustup target add x86_64-unknown-linux-musl
RUN cargo build --release --target=x86_64-unknown-linux-musl

# 2️⃣ Runtime Stage
FROM debian:bullseye-slim

WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/deploy-api /app/deploy-api

RUN chmod +x /app/deploy-api

ARG DEPLOY_USER
ARG DEPLOY_PASS
ARG SSH_HOST

ENV DEPLOY_USER=${DEPLOY_USER}
ENV DEPLOY_PASS=${DEPLOY_PASS}
ENV SSH_HOST=${SSH_HOST}

EXPOSE 8080

CMD ["/app/deploy-api"]
