use std::marker::PhantomData;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::connect_async;
use crate::OrderBookUpdate;
use serde_json::json;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use serde_with::{serde_as, DisplayFromStr};
use crate::connectors::ExchangeQuote;

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
struct WsResponseOrderBookUpdate{
    event: String,
    channel: String,
    data: OrderBookUpdateData
}

#[derive(Deserialize, Debug)]
struct OrderBookUpdateData{
    bids: Vec<ExchangeQuote>,
    asks: Vec<ExchangeQuote>,
}

impl Into<OrderBookUpdate> for WsResponseOrderBookUpdate {
    fn into(self) -> OrderBookUpdate {
        OrderBookUpdate{
            bid_changes: self.data.bids,
            ask_changes: self.data.asks,
            exchange_id: None,
        }
    }
}


#[serde_as]
#[derive(Deserialize, Debug)]
struct QuoteTuple(
    #[serde_as(as = "DisplayFromStr")]
    pub f64,
    #[serde_as(as = "DisplayFromStr")]
    pub f64
);

#[derive(Deserialize)]
struct DiffBookData{
    timestamp: u64,
    bids: Vec<QuoteTuple>,
    asks: Vec<QuoteTuple>,
}

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
        let sub_message = serde_json::to_vec(&sub_message_json).unwrap();
        'reconnection_loop: loop {
            let (mut stream, _) = connect_async(subscription_url.clone()).await.unwrap();
            stream.send(Message::Binary(sub_message.clone())).await.unwrap(); // TODO exception handling
            let sub_confirmation = stream.next().await;
            if sub_confirmation.is_none(){
                println!("expected confirmation message, got none isntead");
                continue 'reconnection_loop
            }
            let sub_confirmation: WsResponseSubscribe = match sub_confirmation.unwrap() {
                Ok(Message::Text(raw_msg)) => serde_json::from_str(&raw_msg).unwrap(),
                other => {
                    println!("unexpected message: {:?}", other);
                    continue 'reconnection_loop
                }
            };
            if sub_confirmation.channel != channel_name || sub_confirmation.event != "bts:subscription_succeeded"{
                println!("subscription failed: {:?}", sub_message);
                continue 'reconnection_loop
            }
            loop {
                let rcv = stream.next().await;
                if rcv.is_none(){
                    continue
                }
                let raw_msg = match rcv.unwrap(){
                    Ok(val) => val,
                    Err(err) => {
                        // TODO Alex: handle errors
                        println!("err: {:?}", err);
                        stream.close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: "Client requested connection close.".into(),
                        }));
                        continue 'reconnection_loop
                    }
                };

                //println!("raw_msg: {}", &raw_msg);

                let bitstamp_message = match raw_msg {
                    Message::Text(msg) =>{
                        let value: WsResponseOrderBookUpdate = serde_json::from_str(&msg).unwrap();
                        value
                    },
                    other => {
                        println!("unexpected response: {}", other);
                        continue
                    }
                };

                //println!("message: {:?}", bitstamp_message);
                let mut order_book_update: OrderBookUpdate = bitstamp_message.into();
                order_book_update.exchange_id = Some(self.exchange_id);
                if let Err(err) = pub_chan.send(order_book_update).await{
                    println!("{}", err);
                    return;
                }
            }
        }
    }
}