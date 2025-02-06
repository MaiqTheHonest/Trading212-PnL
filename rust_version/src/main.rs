use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::error::Error;
use std::fs;
use chrono::DateTime;
use serde::{Deserialize, Deserializer, de};
use std::{thread, time};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut data = Vec::<Order>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut orders = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Order>)

    while cursor != String::from("complete") {    // repeat until process_items() returns cursor as "complete"

        let api_response = call_api(&cursor).await;

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

    println!("{:?}", data);
    println!("fetched a total of {} orders", data.len());
    // println!("{:.3}", data[0].filledQuantity);
    Ok(())
}



// defining structs for json output to be deserialized into (within call_api)
#[derive(Debug, Deserialize)]
struct Items {
    items: Vec<Order>,

}

#[derive(Debug, Deserialize)]
struct Order {
    ticker: String,
    dateCreated: String,

    #[serde(deserialize_with = "deserialize_null_fields")]    // custom deserialize routine to fill null.
    filledQuantity: f32,                                      // happens because .json has implementation for null,
                                                              // but rust doesn't (and doesn't even treat it as a missing field)    <-\\

    status: String

}

fn deserialize_null_fields<'de, D>(deserializer: D) -> Result<f32, D::Error> where D: Deserializer<'de> {    // the routine itself  <-||
    Option::<f32>::deserialize(deserializer).map(|opt| opt.unwrap_or(0.0))
}



// THIS IS DEVELOP BRANCH
async fn call_api(current_cursor: &String) -> Result<Items, Box<dyn Error>> {

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
        Err(format!("API call failed: {}", response.status()).into())
    }
}



fn process_items(orders: Items) -> (String, Vec<Order>) {

    let timestamp = extract_unix(&orders.items);
    let blarg = match timestamp {
        Some(v) => v,                       // if it worked, return unxi timestamp as cursor (blarg)
        None => String::from("complete")    // it it didn't, return "complete" as cursor (blarg)
    };
    eprintln!("{:?}", blarg);
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
