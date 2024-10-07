FROM rust:1.81-alpine AS builder
WORKDIR /usr/src/rdfs
COPY . .
RUN apk add openssl-dev musl-dev
RUN cargo build --release

FROM alpine
COPY --from=builder /usr/src/rdfs/target/release/rdfs /usr/local/bin/rdfs
ENTRYPOINT ["rdfs"]