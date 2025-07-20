#![allow(non_snake_case)]
#![allow(dead_code)]
use std::collections::HashMap;
use std::error::Error;
use chrono::DateTime;
use std::{thread, time};
use crate::t212::{recursive_call_api, extract_unix, CallResponse, Dividend, Dividends, ResponseType};



#[tokio::main]
pub async fn get_dividends(api_key: &str) -> Result<f64, Box<dyn Error>> {

    let mut data = Vec::<Dividend>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut dividends = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Dividend>)

    while cursor != String::from("complete") {    // repeat until process_items() returns cursor as "complete"

        let api_response = recursive_call_api(&api_key, "https://live.trading212.com/api/v0/history/dividends", &cursor, ResponseType::Divis).await;
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
                                                //vvv if none then none, if some then use in this closure  
    let mut timestamp = match dividends.items.last().and_then(|dividend| extract_unix(&dividend.paidOn)) {
        Some(v) => v,                       // if it worked, return unix timestamp as cursor 
        None => String::from("complete")    // it it didn't, return "complete" as cursor
    };
    eprintln!("processed page: {:?}", timestamp);
    

    if dividends.nextPagePath == None {
        timestamp = String::from("complete");
    }
    eprintln!("Dividend import from Trading212: {}", timestamp);
    (timestamp, dividends.items)
}




