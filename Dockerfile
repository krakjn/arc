FROM rust:1.67-alpine
RUN apk add musl-dev
WORKDIR /ci

