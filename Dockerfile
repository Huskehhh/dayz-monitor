FROM rust:latest as builder

WORKDIR /usr/src/dayz-monitor
COPY . .

RUN cargo install --path .

FROM debian:latest

RUN apt-get update && apt-get upgrade -y
RUN apt-get install ca-certificates -y && apt-get install openssl -y 

COPY --from=builder /usr/local/cargo/bin/dayz-monitor /usr/local/bin/dayz-monitor

USER 1000

CMD ["dayz-monitor"]

