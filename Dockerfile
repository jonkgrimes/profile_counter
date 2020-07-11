FROM rust:latest AS builder

RUN apt-get update

RUN apt-get install musl-tools -y

RUN rustup target add x86_64-unknown-linux-musl

WORKDIR /usr/src/profile_counter

COPY . .

RUN RUSTFLAGS=-Clinker=musl-gcc cargo build --release --target=x86_64-unknown-linux-musl

RUN cargo build --release

RUN mkdir -p /build-out/

FROM alpine:latest AS app

COPY --from=builder /usr/src/profile_counter/target/x86_64-unknown-linux-musl/release/profile_counter /usr/local/bin/profile_counter

CMD ["profile_counter"]
