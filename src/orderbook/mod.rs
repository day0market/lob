tonic::include_proto!("orderbook");

use crate::orderbook::orderbook_aggregator_server::OrderbookAggregator;
use tokio::sync::mpsc::channel;
use tokio_stream::wrappers::ReceiverStream;
use tonic;
use tonic::{Request, Response, Status};
use tracing::{error, info};

#[derive(Debug)]
pub struct OrderbookAggregatorPublisher {
    receiver: tokio::sync::watch::Receiver<Summary>,
}

impl OrderbookAggregatorPublisher {
    pub fn new(receiver: tokio::sync::watch::Receiver<Summary>) -> Self {
        Self { receiver }
    }
}

#[tonic::async_trait]
impl OrderbookAggregator for OrderbookAggregatorPublisher {
    type BookSummaryStream = ReceiverStream<Result<Summary, Status>>;

    async fn book_summary(
        &self,
        _request: Request<Empty>,
    ) -> Result<Response<Self::BookSummaryStream>, Status> {
        info!("new book summary subscriber");
        let (tx, rx) = channel(4);
        let mut summary_receiver = self.receiver.clone();

        tokio::spawn(async move {
            while summary_receiver.changed().await.is_ok() {
                let summary = (*summary_receiver.borrow()).clone();
                if summary.bids.is_empty() || summary.asks.is_empty() {
                    continue;
                }
                info!("publish new book summary: {:?}", &summary);
                if let Err(err) = tx.send(Ok(summary)).await {
                    error!("failed to send book summary. err={:?}", err);
                };
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }
}
