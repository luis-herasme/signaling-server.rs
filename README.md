# signaling-server.rs
This is a very simple signaling server written in Rust. It uses [tokio_tungstenite](https://github.com/snapview/tokio-tungstenite) for WebSocket handling and [tokio](https://github.com/tokio-rs/tokio) for async IO.

This project was created to be used with [WASM-P2P](https://github.com/luis-herasme/wasm_p2p), but it can be used with any project that requires a signaling server and follows the same protocol as `WASM-P2P`.

## Usage
To run the server, you need to have Rust installed. You can install it by following the instructions [here](https://www.rust-lang.org/tools/install).

After installing Rust, you can run the server by executing the following command:
```bash
cargo run
```

By default, the server will run on `localhost:8080`. You can change the address and port by setting the `SIGNALING_SERVER_ADDRESS` environment variable.

## .env example
```bash
SIGNALING_SERVER_ADDRESS=localhost:8080
```
