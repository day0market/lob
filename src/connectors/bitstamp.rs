use crate::common::model::{ExchangeQuote, OrderBookUpdate};
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::json;
use serde_with::{serde_as, DisplayFromStr};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tracing::{error, info};

pub struct BitstampOrderBookListener {
    exchange_symbol: String,
    exchange_id: usize,
}

#[derive(Deserialize)]
struct WsResponseSubscribe {
    event: String,
    channel: String,
}

#[derive(Deserialize, Debug)]
struct WsResponseOrderBookUpdate {
    event: String,
    channel: String,
    data: OrderBookUpdateData,
}

#[derive(Deserialize, Debug)]
struct OrderBookUpdateData {
    bids: Vec<ExchangeQuote>,
    asks: Vec<ExchangeQuote>,
}

impl Into<OrderBookUpdate> for WsResponseOrderBookUpdate {
    fn into(self) -> OrderBookUpdate {
        OrderBookUpdate {
            bid_changes: self.data.bids,
            ask_changes: self.data.asks,
            exchange_id: None,
        }
    }
}

#[serde_as]
#[derive(Deserialize, Debug)]
struct QuoteTuple(
    #[serde_as(as = "DisplayFromStr")] pub f64,
    #[serde_as(as = "DisplayFromStr")] pub f64,
);

impl BitstampOrderBookListener {
    pub fn new(pair: &str, exchange_id: usize) -> Self {
        let exchange_symbol = pair.replace("/", "");
        Self {
            exchange_symbol,
            exchange_id,
        }
    }

    pub async fn run(&self, pub_chan: Sender<OrderBookUpdate>) {
        let subscription_url = "wss://ws.bitstamp.net";
        let channel_name = format!("order_book_{}", &self.exchange_symbol.to_lowercase());
        let sub_message_json = json!({
            "event": "bts:subscribe",
            "data": {
            "channel": channel_name
            }
        });

        info!("bitstamp sub_message_json={:?}", &sub_message_json);

        let sub_message = serde_json::to_vec(&sub_message_json).unwrap();
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
            info!("subscribing to bistamp websocket data");

            let (mut stream, _) = match connect_async(subscription_url.clone()).await {
                Ok(val) => val,
                Err(err) => {
                    error!("failed to connect. err={:?}", err);
                    thread::sleep(Duration::from_secs(5));
                    continue 'reconnection_loop;
                }
            };

            if let Err(err) = stream.send(Message::Binary(sub_message.clone())).await {
                error!("failed to send sub message. err={:?}", err);
                continue 'reconnection_loop;
            };

            let sub_confirmation = stream.next().await;
            if sub_confirmation.is_none() {
                error!("expected confirmation message, got none instead");
                continue 'reconnection_loop;
            }

            let sub_confirmation: WsResponseSubscribe = match sub_confirmation.unwrap() {
                Ok(Message::Text(raw_msg)) => serde_json::from_str(&raw_msg).unwrap(),
                other => {
                    error!("unexpected message ={:?}", other);
                    continue 'reconnection_loop;
                }
            };
            if sub_confirmation.channel != channel_name
                || sub_confirmation.event != "bts:subscription_succeeded"
            {
                error!("subscription failed: {:?}", sub_message);
                continue 'reconnection_loop;
            }
            loop {
                let rcv = stream.next().await;
                if rcv.is_none() {
                    continue;
                }
                let raw_msg = match rcv.unwrap() {
                    Ok(val) => val,
                    Err(err) => {
                        error!("failed to receive websocket message. err={:?}", err);
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

                let bitstamp_message = match raw_msg {
                    Message::Text(msg) => {
                        let value: WsResponseOrderBookUpdate = serde_json::from_str(&msg).unwrap();
                        value
                    }
                    other => {
                        error!("unexpected response ={:?}", other);
                        continue;
                    }
                };

                let mut order_book_update: OrderBookUpdate = bitstamp_message.into();
                order_book_update.exchange_id = Some(self.exchange_id);
                if let Err(err) = pub_chan.send(order_book_update).await {
                    error!("can't send update to chan. err={:?}", err);
                    return;
                }
            }
        }
    }
}
