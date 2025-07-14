#![allow(non_snake_case)]
#![allow(dead_code)]
use std::collections::HashMap;
use std::error::Error;
use chrono::DateTime;
use std::{thread, time};
use crate::t212::{recursive_call_api, CallResponse, Dividend, Dividends, ResponseType};



#[tokio::main]
pub async fn get_dividends() -> Result<f64, Box<dyn Error>> {

    let mut data = Vec::<Dividend>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut dividends = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Dividend>)

    while cursor != String::from("complete") {    // repeat until process_items() returns cursor as "complete"

        let api_response = recursive_call_api("https://live.trading212.com/api/v0/history/dividends", &cursor, ResponseType::Divis).await;
        // println!("{:?}", &api_response);

        (cursor, dividends) = match api_response {   // process_items returns a tuple so we catch both cursor
            Ok(CallResponse::Divis(items)) => process_items(items),            // and orders in this match
            _ => {
                // eprintln!("{}", e);               // doesn't assign tuple but breaks loop so compiler doesn't care
                break
            }
        };

        data.append(&mut dividends);

        thread::sleep(time::Duration::from_millis(10))
    };

    
    let mut total_dividends: f64 = 0.0;

    for item in &mut data {
        item.paidOn = item.paidOn.chars().take(10).collect();    // convert date to daily
        total_dividends += item.amount;                          // increase divi
    };

    // multiply by GBP:USD exchange rate as dividends are always GBP for UK accounts

    Ok(total_dividends)
}



fn process_items(dividends: Dividends) -> (String, Vec<Dividend>) {

    let timestamp = extract_unix(&dividends.items);
    let mut blarg = match timestamp {
        Some(v) => v,                       // if it worked, return unxi timestamp as cursor (blarg)
        None => String::from("complete")    // it it didn't, return "complete" as cursor (blarg)
    };
    if dividends.nextPagePath == None {
        blarg = String::from("complete");
    }
    eprintln!("Dividend import from Trading212: {}", blarg);
    (blarg, dividends.items)
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

