# Overview

OBmerge is a tool for simultaneously connecting to multiple crypto exchanges' websockets and extracting the combined orderbook of the best prices, which is then streamed over a gRPC server.

# Usage

Two binaries are provided which may be run using:

1. `cargo run --bin server` will start a server and an associated CLI for controlling the state of the server. Currently the following commands are implemented:
    -  `start <symbol>` will create a thread for each implemented exchange (currently only binance and bitstamp), connecting to its websocket feed and updating the orderbook in a dedicated triple buffer which can be read by the main thread
    - `stop` will end all exchange connections
    - `quit` will end all processes
2. `cargo run --bin client` will start a client which will output to the console the streamed data from the server.

# Issues/Questions
1. It appears that the bitstamp orderbook is updated less frequently than the binance orderbook, probably because it is streamed at a much higher depth. Is there anything I can do to improve this?

# TODO
- refactor code (maybe)

# Remarks
- src/triplebuffer.rs is currently unused. The triple_buffer dependency is used instead.
- the allowed symbols are {`ethbtc`, `ethusdt`, `btcusdt`}
- the binance connection will be valid for 24 hours
- the bitstamp connection will be valid for 90 days