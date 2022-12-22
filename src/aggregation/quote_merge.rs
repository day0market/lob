use crate::common::model::{AggregatedBookQuote, ExchangeQuote};
use std::cmp::Ordering;

pub trait MergeQuotes {
    fn merge_quotes(
        &mut self,
        exchanges_quotes: &[Vec<ExchangeQuote>],
        old_top: &[AggregatedBookQuote],
        reverse_ordering: bool,
    ) -> Option<Vec<AggregatedBookQuote>>;
}

pub struct VecSortMergeQuotes {
    storage: Vec<AggregatedBookQuote>,
    top_book_depth: usize,
}

impl VecSortMergeQuotes {
    pub fn new(top_book_depth: usize, n_exchanges: usize) -> Self {
        let storage_capacity = top_book_depth * n_exchanges;
        let storage = Vec::with_capacity(storage_capacity);
        Self {
            storage,
            top_book_depth,
        }
    }
}

impl MergeQuotes for VecSortMergeQuotes {
    fn merge_quotes(
        &mut self,
        exchanges_quotes: &[Vec<ExchangeQuote>],
        old_top: &[AggregatedBookQuote],
        reverse_ordering: bool,
    ) -> Option<Vec<AggregatedBookQuote>> {
        self.storage.clear();
        'exchange_book_loop: for (exchange_id, exchange_book) in exchanges_quotes.iter().enumerate()
        {
            for (i, quote) in exchange_book.iter().enumerate() {
                if i >= self.top_book_depth {
                    continue 'exchange_book_loop;
                }
                self.storage.push(AggregatedBookQuote {
                    exchange: exchange_id,
                    price: quote.price,
                    qty: quote.qty,
                })
            }
        }

        self.storage.sort();
        if reverse_ordering {
            self.storage.reverse()
        }

        self.storage.truncate(self.top_book_depth);
        let mut top_changed = false;

        if self.storage.len() != old_top.len() {
            top_changed = true
        } else {
            for (new, old) in self.storage.iter().zip(old_top) {
                if !new.eq(old) {
                    top_changed = true;
                    break;
                }
            }
        };

        if top_changed {
            let mut new_top = Vec::with_capacity(self.top_book_depth);
            new_top.append(&mut self.storage);
            return Some(new_top);
        }

        None
    }
}

pub struct IterativeMergeQuotes {
    top_of_book: Vec<AggregatedBookQuote>,
    indexes: Vec<Option<usize>>,
    empty_indexes: Vec<usize>,
    top_book_depth: usize,
    exchanges_number: usize,
}

impl IterativeMergeQuotes {
    pub fn new(top_book_depth: usize, exchanges_number: usize) -> Self {
        let top_of_book = Vec::with_capacity(top_book_depth);
        let indexes = Vec::with_capacity(exchanges_number);
        let empty_indexes = Vec::with_capacity(exchanges_number);
        Self {
            top_of_book,
            indexes,
            empty_indexes,
            top_book_depth,
            exchanges_number,
        }
    }
}

impl MergeQuotes for IterativeMergeQuotes {
    fn merge_quotes(
        &mut self,
        exchanges_quotes: &[Vec<ExchangeQuote>],
        old_top: &[AggregatedBookQuote],
        reverse_ordering: bool,
    ) -> Option<Vec<AggregatedBookQuote>> {
        self.top_of_book.clear();
        self.empty_indexes.clear();
        self.indexes.clear();

        for _ in 0..self.exchanges_number {
            self.indexes.push(Some(0));
        }
        let mut best_value;
        let mut best_value_exchange;
        let mut best_value_quote_index;
        let mut top_book_changed = false;

        let ordering = if reverse_ordering {
            Ordering::Greater
        } else {
            Ordering::Less
        };

        'merge_loop: loop {
            best_value = None;
            best_value_exchange = None;
            best_value_quote_index = None;

            'index_key_loop: for (exchange_key, exchange_quote_index) in
                self.indexes.iter().enumerate()
            {
                let index = match exchange_quote_index {
                    Some(val) => val,
                    None => continue 'index_key_loop,
                };

                let value = match exchanges_quotes.get(exchange_key) {
                    Some(val) => match val.get(*index) {
                        Some(val) => val,
                        None => continue 'index_key_loop,
                    },
                    None => continue 'index_key_loop,
                };
                let agg_quote = AggregatedBookQuote {
                    exchange: exchange_key,
                    price: value.price,
                    qty: value.qty,
                };
                match &best_value {
                    Some(val) => {
                        if agg_quote.cmp(val) == ordering {
                            best_value = Some(agg_quote);
                            best_value_exchange = Some(exchange_key);
                            best_value_quote_index = Some(index.clone())
                        }
                    }
                    None => {
                        best_value = Some(agg_quote);
                        best_value_exchange = Some(exchange_key);
                        best_value_quote_index = Some(index.clone());
                        continue 'index_key_loop;
                    }
                }
            }

            if best_value_exchange.is_none() || best_value_quote_index.is_none() {
                println!("min_key or min_key_index is none");
                continue 'merge_loop;
            }

            let old_value = old_top.get(self.top_of_book.len());
            match old_value {
                None => {
                    top_book_changed = true;
                }
                Some(val) => {
                    if !val.eq(best_value.as_ref().unwrap()) {
                        top_book_changed = true
                    }
                }
            }
            self.top_of_book.push(best_value.unwrap());
            if self.top_of_book.len() >= self.top_book_depth {
                break 'merge_loop;
            }

            let next_index = best_value_quote_index.unwrap() + 1;
            let min_key_val = best_value_exchange.unwrap();
            if next_index >= exchanges_quotes[min_key_val].len() {
                self.indexes[min_key_val] = None;
                self.empty_indexes.push(min_key_val)
            } else {
                self.indexes[min_key_val] = Some(next_index)
            }

            if self.empty_indexes.len() == exchanges_quotes.len() {
                break 'merge_loop;
            }
        }
        if top_book_changed {
            Some(self.top_of_book.clone())
        } else {
            None
        }
    }
}
