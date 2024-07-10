from rust:bookworm as builder

workdir /app
copy . .
run cargo build --release

from debian:bookworm-slim as runner

workdir /app
copy --from=builder /app/target/release/signaling-server /app/signaling-server
cmd ["/app/signaling-server"]
expose 1234
