FROM rust:slim-bullseye as builder

WORKDIR /usr/src/loveletter
COPY . .
RUN apt update 
RUN apt install -y pkg-config openssl libssl-dev
RUN cargo install --path . --profile release --no-default-features -F server

FROM debian:bullseye-slim

COPY --from=builder /usr/local/cargo/bin/loveletter /usr/local/bin/loveletter
COPY --from=builder /usr/src/loveletter/static static
RUN apt update 
RUN apt install -y openssl libssl-dev ca-certificates
RUN rm -rf /var/lib/apt/lists/*

EXPOSE 8080
CMD ["loveletter"]