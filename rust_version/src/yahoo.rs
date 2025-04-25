use core::f64;
use std::collections::HashMap;

use reqwest::{Client, header::USER_AGENT};
use serde_json::Value;
use chrono::{Datelike, Duration, NaiveDate, TimeZone, Utc};


// accepts string slice with ticker passed to it from main

#[tokio::main]
pub async fn get_prices(symbol: &str, start_date: NaiveDate, end_date: NaiveDate) -> Result<HashMap<NaiveDate, f64>, Box<dyn std::error::Error>> {

    // Convert dates to UNIX timestamps
    let start_timestamp = to_unix(start_date - Duration::days(1));    // move start_date back a day in case start_date is today
    let mut end_timestamp = to_unix(end_date);

    if start_timestamp == end_timestamp {
        end_timestamp += -86400;                          // move end_date back a day in case start and end are the same day now
    }
    let url = format!(
        "https://query1.finance.yahoo.com/v8/finance/chart/{}?period1={}&period2={}&interval=1d",
        symbol, start_timestamp, end_timestamp
    );

    let client = Client::new();
    let response = client.get(&url)
        .header(USER_AGENT, "Mozilla/5.0") // Prevents blocking by Yahoo
        .send()
        .await?
        .text()
        .await?;


    let json: Value = serde_json::from_str(&response)?;

    let mut price_range: HashMap<NaiveDate, f64> = HashMap::new();

    // grotesque json unpacking; we take only timestamps and closing prices
    if let Some(timestamps) = json["chart"]["result"][0]["timestamp"].as_array() {
        if let Some(prices) = json["chart"]["result"][0]["indicators"]["quote"][0]["close"].as_array() {

            for (count, timestamp) in timestamps.iter().enumerate() {

                // looks complicated but its just pairwaise matching of price and date arrays
                // returned by yahoo, using tuples
                // vvv

                if let (Some(timestamp), Some(price)) = (timestamp.as_i64(), prices.get(count).and_then(|p| p.as_f64())) {
                    let date = unix_to_date(timestamp);
                    price_range.insert(date, price);
                }
            };
            // println!("{:?}", price_range)
        }
    } else {
        println!("Could not fetch price data for ticker: {}", symbol);
    }

    Ok(price_range)
}




// convert NaiveDate to UNIX timestamp
fn to_unix(date: NaiveDate) -> i64 {
    let datetime = Utc.with_ymd_and_hms(date.year(), date.month(), date.day(), 0, 0, 0).single().unwrap();
    datetime.timestamp()
}




// convert UNIX timestamp to NaiveDate
fn unix_to_date(timestamp: i64) -> NaiveDate {
    Utc.timestamp_opt(timestamp, 0).unwrap().date_naive()
}




pub fn convert_to_yahoo_ticker(
    ticker: String,
    ) -> String {
    
        let pre_dict_tickers = HashMap::from([       // exchange codes
            ("a", "AS"),
            ("d", "DE"),
            ("e", "MC"),
            ("p", "PA"),
            ("l", "L"),
            ("s", "SW"),
            ("m", "MI")
            ]);
        
        let post_dict_tickers = HashMap::from([      // country codes
            ("PT", "LS"),
            ("AT", "VI"),
            ("BE", "BR"),
            ("CA", "TO")
            ]);

    let mut returnable_ticker: String = String::new();

    if let Some(pos) = ticker.rfind("_EQ") {
        let before_eq = &ticker[..pos];                              // take what's before _EQ
        let parts: Vec<&str> = before_eq.split('_').collect();       // separate what's left by _ and turn into collection



        if parts.len() == 1 {
            let mut pre = parts[0];
            let borse = pre.chars().last().unwrap().to_string();

            let y_borse = match pre_dict_tickers.get(&*borse) {    // the most deranged deref usage I've done
                Some(v) => v,
                None => panic!("couldn't find exchange with postfix: {}", &borse)
            }.to_owned();

            pre = &pre[..pre.len() - 1];
            // let corrupt_tickers = vec!["VUAA"];                   // some Milano tickers don't work so we try same stock in Germany 
            // if corrupt_tickers.contains(&pre) {
            //     y_borse = "L";
            // }
            returnable_ticker = format!("{}.{}", pre, y_borse);



        } else if parts.len() == 2 {
            let borse = parts[1];
            if borse == "US" {return parts[0].to_string()}         // if postfix is "US", then no postfix to yahoo ticker is needed


            let y_borse = match post_dict_tickers.get(&*borse) {    
                Some(v) => v,
                None => panic!("couldn't find exchange with postfix: {}", &borse)
            };
            returnable_ticker = format!("{}.{}", parts[0], y_borse);
        } else {}
    };

    returnable_ticker
}


