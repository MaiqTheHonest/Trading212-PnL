use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::error::Error;
use std::fs;
use chrono::DateTime;
use serde::{Deserialize, Deserializer, de};
use std::{thread, time};
use ndarray::{Array1, arr1};


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut data = Vec::<Orders>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut orders = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Orders>)

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

    
    

    for item in &mut data {
        item.dateCreated = item.dateCreated.chars().take(10).collect();    // convert date to daily
    }



    println!("{:?}", data);
    println!("fetched a total of {} orders", data.len());
    // println!("{:?}", data.to_ndarray());
    Ok(())
}



// defining structs for json output to be deserialized into (within call_api)
#[derive(Debug, Deserialize)]
struct Items {
    items: Vec<Orders>,

}

#[derive(Debug, Deserialize)]
struct Orders {
    id: u64,
    ticker: String,
    dateCreated: String,

    #[serde(deserialize_with = "deserialize_null_fields")]    // custom deserialize routine to fill occasional nulls.
    filledQuantity: f32,                                      // happens because .json has implementation for null,
                                                              
    #[serde(deserialize_with = "deserialize_null_fields")]    // but rust doesn't (and doesn't even treat it as a missing field)    <-\\
    fillPrice: f32,


    status: String

}

fn deserialize_null_fields<'de, D>(deserializer: D) -> Result<f32, D::Error> where D: Deserializer<'de> {    // the routine itself  <-||
    Option::<f32>::deserialize(deserializer).map(|opt| opt.unwrap_or(0.0))
}



// impl Orders {
    
//     fn to_ndarray(&self) -> Array1::<f32> {
        
//         arr1(&[self.filledQuantity])
//     }
// }




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



fn process_items(orders: Items) -> (String, Vec<Orders>) {

    let timestamp = extract_unix(&orders.items);
    let blarg = match timestamp {
        Some(v) => v,                       // if it worked, return unxi timestamp as cursor (blarg)
        None => String::from("complete")    // it it didn't, return "complete" as cursor (blarg)
    };
    eprintln!("{:?}", blarg);
    (blarg, orders.items)
}



fn extract_unix(timestamp: &Vec<Orders>) -> Option<String> {
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



// fn extract_DDMMYYYY(timestamp: &Vec<Orders>) -> String {}