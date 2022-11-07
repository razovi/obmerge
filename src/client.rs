use obmerge::orderbook::OrderBook;
use obmerge::proto::order_book::Empty;
use obmerge::proto::order_book::orderbook_aggregator_client::OrderbookAggregatorClient; 
use serde_json;
use tonic::Request;
use tonic::transport::Channel;

async fn streaming_ob(client: &mut OrderbookAggregatorClient<Channel>) {
    let mut stream = client
        .book_summary(Request::new(Empty{}))
        .await
        .unwrap()
        .into_inner();

    // stream is infinite - take just 5 elements and then disconnect
    loop{
        let item = stream.message().await.unwrap().unwrap();
        println!("\treceived: {}", serde_json::to_string(&OrderBook::from_summary(item)).unwrap());
    }
    // stream is droped here and the disconnect info is send to server
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{

    let mut client = OrderbookAggregatorClient::connect("http://[::1]:50051").await.unwrap();
    streaming_ob(&mut client).await;
    Ok(())
}