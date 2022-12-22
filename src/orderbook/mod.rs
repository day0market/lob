tonic::include_proto!("orderbook");

use crate::orderbook::orderbook_aggregator_server::OrderbookAggregator;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic;
use tonic::{Request, Response, Status};

#[derive(Debug)]
pub struct OrderBookServer {
    receiver: tokio::sync::watch::Receiver<Summary>,
}

impl OrderBookServer {
    pub fn new(receiver: tokio::sync::watch::Receiver<Summary>) -> Self {
        Self { receiver }
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderBookServer {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        let (mut tx, rx) = channel(4);
        let mut summary_receiver = self.receiver.clone();

        tokio::spawn(async move {
            while summary_receiver.changed().await.is_ok() {
                let summary = (*summary_receiver.borrow()).clone();
                if summary.bids.is_empty() || summary.asks.is_empty() {
                    continue;
                }
                tx.send(Ok(summary)).await.unwrap();
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
