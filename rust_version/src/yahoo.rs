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

