FROM rust:latest AS builder

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /app

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN mkdir -p /build-out/

FROM alpine:latest AS app

RUN mkdir -p /app/static

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/profile_counter /app/profile_counter
COPY --from=builder /app/static/. /app/static/.

CMD ["/app/profile_counter"]
