FROM rust:1.96-alpine
RUN apk add musl-dev
WORKDIR /ci
