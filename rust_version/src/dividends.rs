#![allow(non_snake_case)]
#![allow(dead_code)]
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::error::Error;
use std::fs;
use chrono::DateTime;
use serde::Deserialize;
use std::{thread, time};
use crate::stats::GBPUSD; // change this to dynamic FX


#[tokio::main]
pub async fn get_dividends() -> Result<f64, Box<dyn Error>> {

    let mut data = Vec::<Dividend>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut orders = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Dividend>)

    while cursor != String::from("complete") {    // repeat until process_items() returns cursor as "complete"

        let api_response = call_api(&cursor).await;
        // println!("{:?}", &api_response);

        (cursor, orders) = match api_response {   // process_items returns a tuple so we catch both cursor
            Ok(v) => process_items(v),            // and orders in this match
            Err(e) => {
                eprintln!("{}", e);               // doesn't assign tuple but breaks loop so compiler doesn't care
                break
            }
        };

        data.append(&mut orders);

        thread::sleep(time::Duration::from_millis(10))
    };

    
    let mut total_dividends: f64 = 0.0;

    for item in &mut data {
        item.paidOn = item.paidOn.chars().take(10).collect();    // convert date to daily
        total_dividends += item.amount;                          // increase divi
    };

    // multiply by GBP:USD exchange rate as dividends are always GBP for UK accounts
    Ok(total_dividends*GBPUSD)
}



// defining structs for json output to be deserialized into (within call_api)
#[derive(Debug, Deserialize)]
struct Items {
    items: Vec<Dividend>,
    nextPagePath: Option<String>

}

#[derive(Debug, Deserialize)]
pub struct Dividend {                                            // both the struct and fields have to be public to be accessed in main
    pub ticker: String,
    pub amount: f64,
    pub paidOn: String
}


// THIS IS DEVELOP BRANCH
async fn call_api(current_cursor: &String) -> Result<Items, Box<dyn Error>> {

    let api_key = fs::read_to_string("api_key.txt")
    .expect("could not find api_key.txt");

    let api_url = "https://live.trading212.com/api/v0/history/dividends";

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);

    let params = HashMap::from([
        ("cursor", current_cursor.as_str()),
        ("ticker", ""),
        ("limit", "40")]);

    let client = reqwest::Client::new();
    let response = client
        .get(api_url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let catcher: Items = response.json().await?;    // Items is the outer struct to which we feed serde_json output
        Ok(catcher)

    } else {
        Err(format!("API call failed: {}", response.status()).into())
    }
}



fn process_items(orders: Items) -> (String, Vec<Dividend>) {

    let timestamp = extract_unix(&orders.items);
    let mut blarg = match timestamp {
        Some(v) => v,                       // if it worked, return unxi timestamp as cursor (blarg)
        None => String::from("complete")    // it it didn't, return "complete" as cursor (blarg)
    };
    if orders.nextPagePath == None {
        blarg = String::from("complete");
    }
    eprintln!("Dividend import from Trading212: {}", blarg);
    (blarg, orders.items)
}



fn extract_unix(timestamp: &Vec<Dividend>) -> Option<String> {
    // shadowing
    let timestamp = timestamp
    .last()?
    .paidOn
    .as_str();

    let timestamp = DateTime::parse_from_rfc3339(timestamp)
    .ok()?
    .timestamp_millis()
    .to_string();

    Some(timestamp)
}

