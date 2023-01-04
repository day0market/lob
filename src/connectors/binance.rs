use crate::common::model::{ExchangeQuote, OrderBookUpdate};
use flate2::read::GzDecoder;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json;
use std::io::Read;
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::{
    connect_async,
    tungstenite::protocol::{frame::coding::CloseCode, CloseFrame},
    tungstenite::Message,
};
use tracing::{error, info};

pub struct BinanceOrderBookListener {
    exchange_symbol: String,
    exchange_id: usize,
}

//{
//   "lastUpdateId": 160,  // Last update ID
//   "bids": [             // Bids to be updated
//     [
//       "0.0024",         // Price level to be updated
//       "10"              // Quantity
//     ]
//   ],
//   "asks": [             // Asks to be updated
//     [
//       "0.0026",         // Price level to be updated
//       "100"             // Quantity
//     ]
//   ]
// }

#[derive(Deserialize, Debug)]
struct BinanceOrderBookUpdate {
    bids: Vec<ExchangeQuote>,
    asks: Vec<ExchangeQuote>,
}

impl Into<OrderBookUpdate> for BinanceOrderBookUpdate {
    fn into(self) -> OrderBookUpdate {
        OrderBookUpdate {
            bid_changes: self.bids,
            ask_changes: self.asks,
            exchange_id: None,
        }
    }
}

impl BinanceOrderBookListener {
    pub fn new(pair: &str, exchange_id: usize) -> Self {
        let exchange_symbol = pair.replace("/", "");
        Self {
            exchange_symbol,
            exchange_id,
        }
    }
    pub async fn run(&self, pub_chan: Sender<OrderBookUpdate>) {
        let subscription_url = format!(
            "wss://stream.binance.com:443/ws/{}@depth{}@100ms",
            &self.exchange_symbol.to_lowercase(),
            20
        );

        'reconnection_loop: loop {
            {
                // Send empty bid and asks to reset collector quotes
                let order_book_update = OrderBookUpdate {
                    exchange_id: Some(self.exchange_id),
                    bid_changes: vec![],
                    ask_changes: vec![],
                };

                if let Err(err) = pub_chan.send(order_book_update).await {
                    error!("can't send update to chan. err={:?}", err);
                    return;
                }
            }
            thread::sleep(Duration::from_secs(1)); // prevents ws spamming
            info!("subscribing to binance websocket data");
            let (mut stream, _) = match connect_async(subscription_url.clone()).await {
                Ok(val) => val,
                Err(err) => {
                    error!("failed to connect. err={:?}", err);
                    thread::sleep(Duration::from_secs(5));
                    continue 'reconnection_loop;
                }
            };

            loop {
                let rcv = stream.next().await;
                if rcv.is_none() {
                    continue;
                }
                let raw_msg = match rcv.unwrap() {
                    Ok(val) => val,
                    Err(err) => {
                        error!("error in websocket recv {:?}", err);
                        if let Err(err) = stream
                            .close(Some(CloseFrame {
                                code: CloseCode::Normal,
                                reason: "Client requested connection close.".into(),
                            }))
                            .await
                        {
                            error!("can't close websocket err={:?}", err);
                        };
                        continue 'reconnection_loop;
                    }
                };

                let binance_message = match raw_msg {
                    Message::Text(raw_msg) => {
                        let value: BinanceOrderBookUpdate = serde_json::from_str(&raw_msg).unwrap();
                        value
                    }
                    Message::Binary(raw_msg) => {
                        let mut d = GzDecoder::new(&*raw_msg);
                        let mut s = String::new();
                        d.read_to_string(&mut s).unwrap();
                        let value: BinanceOrderBookUpdate = serde_json::from_str(&s).unwrap();
                        value
                    }
                    Message::Ping(_) => {
                        if let Err(err) = stream.send(Message::Pong("pong".into())).await {
                            error!("failed to send pong. err={:?}", err)
                        };
                        continue;
                    }
                    other => {
                        error!("received unexpected message={:?}", other);
                        continue;
                    }
                };

                let mut order_book_update: OrderBookUpdate = binance_message.into();
                order_book_update.exchange_id = Some(self.exchange_id);
                if let Err(err) = pub_chan.send(order_book_update).await {
                    error!("can't send update to chan. err={:?}", err);
                    return;
                }
            }
        }
    }
}
