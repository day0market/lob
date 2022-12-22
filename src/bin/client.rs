use futures_util::StreamExt;
use lob::orderbook::orderbook_aggregator_client::OrderbookAggregatorClient;
use lob::orderbook::Empty;
use tonic::transport::Endpoint;
use tonic::Request;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let addr = Endpoint::from_static("https://127.0.0.1:50051");

    let mut client = OrderbookAggregatorClient::connect(addr).await?;
    let request = Request::new(Empty {});
    let mut response = client.book_summary(request).await.unwrap().into_inner();

    while let Some(row) = response.next().await {
        println!("new: {:#?}", &row);
    }

    Ok(())
}
