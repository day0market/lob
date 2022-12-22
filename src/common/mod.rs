pub mod model;

#[cfg(test)]
mod tests{
    use std::cmp::Ordering;
    use super::model::AggregatedBookQuote;

    #[test]
    fn aggregated_quote_cmp(){
        {
            let q_bigger = AggregatedBookQuote{
                exchange: 0,
                price: 1.001,
                qty: 2.0
            };

            let q_smaller = AggregatedBookQuote{
                exchange: 2,
                price: 1.00,
                qty: 2.0
            };

            assert_eq!(q_bigger.cmp(&q_smaller), Ordering::Greater);
        }


        {
            let q_bigger = AggregatedBookQuote{
                exchange: 0,
                price: 1.001,
                qty: 2.5
            };
            let q_smaller = AggregatedBookQuote{
                exchange: 2,
                price: 1.001,
                qty: 2.0
            };
            assert_eq!(q_bigger.cmp(&q_smaller), Ordering::Greater);
        }

        {
            let q_bigger = AggregatedBookQuote{
                exchange: 2,
                price: 1.001,
                qty: 2.5
            };
            let q_smaller = AggregatedBookQuote{
                exchange: 0,
                price: 1.001,
                qty: 2.5
            };
            assert_eq!(q_bigger.cmp(&q_smaller), Ordering::Greater);
        }


    }
}