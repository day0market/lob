use std::cmp::Ordering;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

#[serde_as]
#[derive(Deserialize, Debug, Serialize, Clone)]
pub struct ExchangeQuote{
    #[serde_as(as = "DisplayFromStr")]
    pub price: f64,
    #[serde_as(as = "DisplayFromStr")]
    pub qty: f64,
}


#[derive(Deserialize, Debug, Serialize)]
pub struct OrderBookUpdate{
    pub exchange_id: Option<usize>,
    pub bid_changes: Vec<ExchangeQuote>,
    pub ask_changes: Vec<ExchangeQuote>,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct AggregatedBookQuote{
    pub exchange: usize,
    pub price: f64,
    pub qty: f64,
}

impl Eq for AggregatedBookQuote {

}




impl Ord for AggregatedBookQuote {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.price.partial_cmp(&other.price){
            Some(Ordering::Equal) | None => {},
            Some(val) => return val,
        };

        match self.qty.partial_cmp(&other.qty) {
            Some(Ordering::Equal) | None => {},
            Some(val) => return val,
        };

        self.exchange.cmp(&other.exchange)
    }
}