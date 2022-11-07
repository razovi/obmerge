# Overview

OBmerge is a tool for simultaneously connecting to multiple crypto exchanges' websockets and extracting the combined orderbook of the best prices, which is then streamed over a gRPC server.

# Usage

Running this tool will start a CLI for controlling the state of the server. Currently the following commands are implemented:

- `start <symbol>` will create a thread for each implemented exchange (currently only binance and bitstamp), connecting to its websocket feed and updating the orderbook in a dedicated triple buffer which can be read by the main thread
- `stop` will end all connections
- `check` will read the current orderbooks, create the best combined order book from 10 bids/asks, and print it
- `quit` will end all connnections and end the program's execution

# Issues/Questions
1. Currently I'm having a lot of issues making the gRPC server work, as I do not have much experience in this area. My problem is that anything I try to do seems to require that the book_summary function changes the state of the server, however the proto file only gives this function a non-mutable reference to the server. Also, every example I found online doesn't deal with a server with a changing internal state, so I am starting to wonder if it is even possible.
2. It appears that the bitstamp orderbook is updated less frequently than the binance orderbook, probably because it is streamed at a much higher depth. Is there anything I can do to improve this?

# TODO
- Fix/Implement the gRPC server.
- Trying to monitor an inexistent symbol will currently allow the connection. Add a list of allowed symbols to prevent this.
- accept multiple types of messages from exchanges
- refactor code
- better error handling
- add POST messages to stop disconnections
- make the combined orderbook update on every change of any orderbook, not on request

# Remarks
- src/triplebuffer.rs is currently unused. The triple_buffer dependency is used instead.