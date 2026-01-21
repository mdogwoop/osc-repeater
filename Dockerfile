FROM rust:alpine AS builder
RUN apk add --no-cache musl-dev
WORKDIR /usr/src/osc-repeater
COPY . .
RUN cargo build --release --bin osc-repeater

FROM alpine
RUN apk add --no-cache ca-certificates
COPY --from=builder /usr/src/osc-repeater/target/release/osc-repeater /usr/local/bin/osc-repeater
ENTRYPOINT ["/usr/local/bin/osc-repeater"]
