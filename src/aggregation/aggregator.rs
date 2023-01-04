use crate::aggregation::quote_merge::MergeQuotes;
use crate::common::model::OrderBookUpdate;
use crate::common::model::{AggregatedBookQuote, ExchangeQuote};
use crate::orderbook::{Level, Summary};
use std::collections::HashMap;
use tracing::error;

pub struct OrderBookAggregator<T: MergeQuotes> {
    exchanges_bids: Vec<Vec<ExchangeQuote>>,
    exchanges_asks: Vec<Vec<ExchangeQuote>>,
    bid_book_top: Vec<AggregatedBookQuote>,
    ask_book_top: Vec<AggregatedBookQuote>,

    exchanges_number: usize,
    top_book_depth: usize,
    quotes_merger: T,
    exchanges_id_mapping: HashMap<usize, String>,
}

impl<T: MergeQuotes> OrderBookAggregator<T> {
    pub fn new(
        quotes_merger: T,
        exchanges_number: usize,
        top_book_depth: usize,
        exchanges_id_mapping: HashMap<usize, String>,
    ) -> Self {
        let mut exchanges_bids = Vec::with_capacity(exchanges_number);
        let mut exchanges_asks = Vec::with_capacity(exchanges_number);
        for _ in 0..exchanges_number {
            exchanges_bids.push(Vec::with_capacity(top_book_depth));
            exchanges_asks.push(Vec::with_capacity(top_book_depth));
        }

        let old_bid_book_top = Vec::with_capacity(top_book_depth.into());
        let old_ask_book_top = Vec::with_capacity(top_book_depth.into());

        Self {
            exchanges_bids,
            exchanges_asks,
            bid_book_top: old_bid_book_top,
            ask_book_top: old_ask_book_top,
            exchanges_number,
            top_book_depth,
            quotes_merger,
            exchanges_id_mapping,
        }
    }

    pub fn process(&mut self, order_book_update: OrderBookUpdate) -> Option<Summary> {
        let exchange_id = match order_book_update.exchange_id {
            Some(val) => val,
            None => {
                error!("exchange_id is empty. skip update");
                return None;
            }
        };

        let mut top_changed = false;

        self.exchanges_asks[exchange_id] = order_book_update.ask_changes;
        self.exchanges_bids[exchange_id] = order_book_update.bid_changes;

        if let Some(val) =
            self.quotes_merger
                .merge_quotes(&self.exchanges_bids, &self.bid_book_top, true)
        {
            top_changed = true;
            self.bid_book_top = val;
        }

        if let Some(val) =
            self.quotes_merger
                .merge_quotes(&self.exchanges_asks, &self.ask_book_top, false)
        {
            top_changed = true;
            self.ask_book_top = val;
        };

        if top_changed {
            return self.get_summary();
        };

        None
    }

    fn get_summary(&self) -> Option<Summary> {
        if self.bid_book_top.is_empty() || self.ask_book_top.is_empty() {
            return None;
        }

        let mut bids = Vec::with_capacity(self.top_book_depth);
        let mut asks = Vec::with_capacity(self.top_book_depth);

        for bid in &self.bid_book_top {
            bids.push(Level {
                exchange: self
                    .exchanges_id_mapping
                    .get(&bid.exchange)
                    .unwrap()
                    .clone(),
                price: bid.price,
                amount: bid.qty,
            })
        }

        for ask in &self.ask_book_top {
            asks.push(Level {
                exchange: self
                    .exchanges_id_mapping
                    .get(&ask.exchange)
                    .unwrap()
                    .clone(),
                price: ask.price,
                amount: ask.qty,
            })
        }

        let spread = self.ask_book_top[0].price - self.bid_book_top[0].price;

        Some(Summary { spread, bids, asks })
    }
}
