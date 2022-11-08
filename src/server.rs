use futures::Stream;
use std::{pin::Pin};
use tokio::sync::{broadcast, broadcast::error::RecvError, mpsc};
use tonic::{transport::Server, Response, Status};
use crate::proto::order_book::orderbook_aggregator_server::{OrderbookAggregator, OrderbookAggregatorServer};
use crate::proto::order_book::{Summary, Empty};
use std::net::ToSocketAddrs;


#[derive(Debug)]
pub struct BookServer {
    tx: broadcast::Sender<Option<Summary>>
}

#[tonic::async_trait]
impl OrderbookAggregator for BookServer {
    type BookSummaryStream = Pin<Box<dyn Stream<Item = Result<Summary, Status>> + Send>>;
    async fn book_summary(&self, _: tonic::Request<Empty>) -> Result<Response<Self::BookSummaryStream>, Status> {
        let mut rx = self.tx.subscribe();
        
        let (ctx, crx) = mpsc::channel::<Result<Summary, Status>>(1);
       tokio::spawn (async move {
            loop{
                let data = rx.recv().await;
                match data {
                    Ok(x) => {
                        if let Some(y) = x {
                            match ctx.send(Ok(y)).await {
                                _ => {}
                            }
                        }
                        else{
                            break;
                        }
                    }
                    Err(error) => {
                        match error {
                            RecvError::Lagged(_) => {
                                println!("Lagged");
                            }
                            _ => {
                                panic!("Unexpected error");
                            }
                        }
                    }
                    
                }
            }
        });

        Ok(Response::new(Box::pin(
            tokio_stream::wrappers::ReceiverStream::new(crx),
        )))
        
    }
}

pub async fn start(tx: broadcast::Sender<Option<Summary>>) -> Result<(), Box<dyn std::error::Error>>{

    let server = BookServer{tx};
    Server::builder()
        .add_service(OrderbookAggregatorServer::new(server))
        .serve("[::1]:50051".to_socket_addrs().unwrap().next().unwrap())
        .await?;
    Ok(())
}