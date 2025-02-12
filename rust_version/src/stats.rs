use std::error::Error;
use chrono::{NaiveDate, Duration};
use std::{collections::{hash_map::Entry, HashMap}, error::Error, str::FromStr};

fn calculate_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>

    ) -> HashMap<NaiveDate, f64> {
    
        
    let mut scoped_portfolio = portfolio_history[0].1;

    for portfolio in portfolio_history {
        scoped_portfolio = match portfolio[0].1 {
            Some(v) => v,
            None => scoped_portfolio
        };
    }

    }