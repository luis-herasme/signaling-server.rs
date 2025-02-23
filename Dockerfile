FROM rust:1.84.0-alpine AS builder

WORKDIR /app
COPY . .
RUN apk add --no-cache musl-dev
RUN cargo build --release

FROM rust:1.84.0-alpine AS runner

WORKDIR /app
COPY --from=builder /app/target/release/signaling-server /app/signaling-server
CMD ["/app/signaling-server"]
EXPOSE 1234
