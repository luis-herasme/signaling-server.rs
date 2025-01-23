FROM rust:slim AS builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM alpine AS runner

WORKDIR /app
COPY --from=builder /app/target/release/signaling-server /app/signaling-server
CMD ["/app/signaling-server"]
EXPOSE 1234
