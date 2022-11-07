use crate::orderbook::OrderBook;

pub mod order_book {
    tonic::include_proto!("orderbook");
}

use futures::Stream;
use futures::stream;
use std::sync::{Arc, Mutex};
use std::{net::ToSocketAddrs, pin::Pin};
use crossbeam_channel::Receiver;
use tokio_stream::{wrappers::ReceiverStream};
use tonic::{transport::Server, Request, Response, Status, Streaming};
use order_book::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};

use order_book::{Empty, Summary, Level};

type EchoResult<T> = Result<Response<T>, Status>;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;

/*#[derive(Debug)]
pub struct EchoServer {
    messages: Arc<Mutex<Vec<Summary>>>
}

#[tonic::async_trait]
impl OrderbookAggregator for EchoServer {
    type BookSummaryStream = ResponseStream;
    async fn book_summary(&self, _: tonic::Request<order_book::Empty>) -> Result<tonic::Response<Self::BookSummaryStream>, tonic::Status> {
        let mut x = self.messages.lock().unwrap();
        let mut cv = Vec::new();
        for book in *x {
            cv.push(book);
        }
        x.clear();
        let stream = stream::iter(cv);
        /*Ok(Response::new(
            Box::pin())
        )*/

    }
}
*/

#[tokio::main]
async fn start(rx: Receiver<OrderBook>){
    /*let mut data = Arc::new(Mutex::new(Vec::new()));
    let mut server = EchoServer {messages: Arc::clone(data)};
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(server))
        .serve("[::1]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await
        .unwrap();
    loop{
        let x = rx.recv();
        data.lock().unwrap().push(x.unwrap().toSummary());
    }*/
}
