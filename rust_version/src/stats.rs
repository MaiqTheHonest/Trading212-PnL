use chrono::NaiveDate;
use std::collections::HashMap;

pub fn calculate_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>

    ) -> Option<HashMap<NaiveDate, f64>> {

    let mut return_history: HashMap<NaiveDate, f64> = HashMap::new();
    let mut portfolio = portfolio_history[0].1.clone();
    // println!("{:?}", portfolio_history);
    let mut outer_holder_value: f64 = 0.0;
    let mut outer_holder_sum: f64 = 0.0;

    for tupleobject in &portfolio_history {
        let date = tupleobject.0;
        if !tupleobject.1.is_empty() {
            portfolio = tupleobject.1.clone();
        }
    
        let mut value_total = 0.0;
        let mut sum_of_mid_returns = 0.0;
        let mut has_valid_data = false;
    
        let mut tickers: Vec<_> = portfolio.keys().collect();
        tickers.sort();
    
        for ticker in tickers {
            if let Some(single_history) = complete_prices.get(ticker) {
                if let Some(p_1) = single_history.get(&date) {
                    let (q_0, p_0) = portfolio[ticker];
                    let mid_return = q_0 * p_0 * (p_1 / p_0 - 1.0);
                    value_total += q_0 * p_0;
                    sum_of_mid_returns += mid_return;
                    has_valid_data = true;
                }
            }
        }
    
        if has_valid_data {
            outer_holder_value = value_total;
            outer_holder_sum = sum_of_mid_returns;
        } else {
            value_total = outer_holder_value;
            sum_of_mid_returns = outer_holder_sum;
        }
    
        if value_total != 0.0 {
            let daily_return = sum_of_mid_returns / value_total;
            return_history.insert(date, daily_return);
        }
    }
    Some(return_history)
} // if could get it, do as normal. if couldn't get it, last line + break
