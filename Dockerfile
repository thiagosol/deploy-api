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

RUN apt update && apt install -y openssh-client

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/deploy-api /app/deploy-api

RUN chmod +x /app/deploy-api

ARG DEPLOY_USER
ARG DEPLOY_PASS
ARG SERVER_IP
ARG SERVER_USERNAME

ENV DEPLOY_USER=${DEPLOY_USER}
ENV DEPLOY_PASS=${DEPLOY_PASS}
ENV SSH_HOST=${SERVER_IP}
ENV SSH_USER=${SERVER_USERNAME}

EXPOSE 8080

CMD ["/app/deploy-api"]
