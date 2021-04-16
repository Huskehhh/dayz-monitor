FROM rust:1.50.0 as builder

WORKDIR /usr/src/dayz-monitor
COPY . .

RUN cargo install --path .

FROM debian:latest

RUN apt-get update && apt-get upgrade -y

COPY --from=builder /usr/local/cargo/bin/dayz-monitor /

CMD ["./dayz-monitor"]