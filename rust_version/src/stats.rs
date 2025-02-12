use std::error::Error;
use chrono::{NaiveDate, Duration};
use std::{collections::{hash_map::Entry, HashMap}, error::Error, str::FromStr};

fn calculate_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>

    ) -> HashMap<NaiveDate, f64> {
    // hello world
    }