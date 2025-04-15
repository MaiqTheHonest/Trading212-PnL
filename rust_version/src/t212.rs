#![allow(non_snake_case)]
#![allow(dead_code)]
use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use std::error::Error;
use std::fs;
use chrono::DateTime;
use serde::{Deserialize, Deserializer};
use std::{thread, time::Duration};
// use futures::future::{BoxFuture, FutureExt, Future};
// use std::pin::Pin;


#[tokio::main]
pub async fn get_orders() -> Result<Vec<Order>, Box<dyn Error>> {

    let mut data = Vec::<Order>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut orders = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Order>)

    while cursor != String::from("complete") {    // repeat until process_items() returns cursor as "complete"

        let api_response = recursive_call_api(&cursor).await;
        // println!("{:?}", api_response);

        (cursor, orders) = match api_response {   // process_items returns a tuple so we catch both cursor
            Ok(v) => process_items(v),            // and orders in this match
            Err(e) => {
                eprintln!("{}", e);               // doesn't assign tuple but breaks loop so compiler doesn't care
                break
            }
        };
        data.append(&mut orders);
        
        // thread::sleep(time::Duration::from_millis(10))
    };

    
    

    for item in &mut data {
        item.dateCreated = item.dateCreated.chars().take(10).collect();    // convert date to daily
    }


    println!("\nfetched a total of {} orders", data.len());

    Ok(data)
}



// defining structs for json output to be deserialized into (within call_api)
#[derive(Debug, Deserialize)]
struct Items {
    items: Vec<Order>,

}

#[derive(Debug, Deserialize)]
pub struct Order {                                            // both the struct and fields have to be public to be accessed in main
    pub id: u64,
    pub ticker: String,
    pub dateCreated: String,

    #[serde(deserialize_with = "deserialize_null_fields")]    // custom deserialize routine to fill occasional nulls.
    pub filledQuantity: f64,                                  // happens because .json has implementation for null,
                                                              
    #[serde(deserialize_with = "deserialize_null_fields")]    // but rust doesn't (and doesn't even treat it as a missing field)    <-\\
    pub fillPrice: f64,

    #[serde(deserialize_with = "deserialize_null_fields")]    
    pub filledValue: f64,

    pub status: String

}

fn deserialize_null_fields<'de, D>(deserializer: D) -> Result<f64, D::Error> where D: Deserializer<'de> {    // the routine itself  <-||
    Option::<f64>::deserialize(deserializer).map(|opt| opt.unwrap_or(0.0))
}

// THIS IS DEVELOP BRANCH


async fn recursive_call_api(current_cursor: &String) -> Result<Items, Box<dyn Error>>{

    let api_key = fs::read_to_string("api_key.txt")
    .expect("could not find api_key.txt");

    let api_url = "https://live.trading212.com/api/v0/equity/history/orders";

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);

    let params = HashMap::from([
        ("cursor", current_cursor.as_str()),
        ("ticker", ""),
        ("limit", "50")]);

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
        if response.status().as_str().contains("429"){  // 429 means too many requests
            countdown(60);
            let d2_response = Box::pin(recursive_call_api(current_cursor)).await;  // Box::pin because Rust doesn't allow recursive async funcs that are not boxed
            return d2_response
        } else {
            Err(format!("API call failed: {}", response.status()).into())
        }
    }
}




fn process_items(orders: Items) -> (String, Vec<Order>) {

    let timestamp = extract_unix(&orders.items);
    let blarg = match timestamp {
        Some(v) => v,                       // if it worked, return unxi timestamp as cursor (blarg)
        None => String::from("complete")    // it it didn't, return "complete" as cursor (blarg)
    };
    eprintln!("processed page: {:?}", blarg);
    (blarg, orders.items)
}



fn extract_unix(timestamp: &Vec<Order>) -> Option<String> {
    // shadowing
    let timestamp = timestamp
    .last()?
    .dateCreated
    .as_str();

    let timestamp = DateTime::parse_from_rfc3339(timestamp)
    .ok()?
    .timestamp_millis()
    .to_string();

    Some(timestamp)
}



fn countdown(mut seconds: i32){
    while seconds > 0 {
        print!("\rAPI rate limit exceeded, further orders fetched automatically in {}", seconds);
        std::io::Write::flush(&mut std::io::stdout()).unwrap(); // flush last line
        thread::sleep(Duration::from_secs(1));
        seconds -= 1;
    }
    std::io::Write::flush(&mut std::io::stdout()).unwrap();     // flush it again at the end
    println!("");
}