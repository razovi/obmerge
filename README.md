# Overview

OBmerge is a tool for simultaneously connecting to multiple crypto exchanges' websockets and extracting the combined orderbook of the best prices, which is then streamed over a gRPC server.

# Usage

Two binaries are provided:

1. `server.exe` will start a server and an associated CLI for controlling the state of the server. Currently the following commands are implemented:
    -  `start <symbol>` will create a thread for each implemented exchange (currently only binance and bitstamp), connecting to its websocket feed and updating the orderbook in a dedicated triple buffer which can be read by the main thread
    - `stop` will end all exchange connections
    - `quit` will end all processes
2. `client.exe` will start a client which will output to the console the streamed data from the server.

These binaries may also be built using:
1. `cargo build --release --bin server`
2. `cargo build --release --bin client`
# Remarks
- src/triplebuffer.rs is currently unused. The triple_buffer dependency is used instead.
- the allowed symbols are {`ethbtc`, `ethusdt`, `btcusdt`}
- the binance connection will be valid for 24 hours
- the bitstamp connection will be valid for 90 days