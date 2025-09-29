#![allow(non_snake_case)]
#![allow(dead_code)]
use std::collections::HashMap;
use reqwest::{header::{HeaderMap, HeaderValue, AUTHORIZATION}, Response};
use std::error::Error;
use chrono::DateTime;
use serde::{Deserialize, Deserializer};
use std::{thread, time::Duration};
use serde_json::Value;



#[tokio::main]
pub async fn get_orders(api_key: &str) -> Result<Vec<Order>, Box<dyn Error>> {

    let mut data = Vec::<Order>::new();
    let mut cursor = String::from("");    // start with empty cursor
    let mut orders = Vec::new();          // and empty vector <T> (holds any type but mine is Vec<Order>)

    while cursor != String::from("complete") {    // repeat until process_items() returns cursor as "complete"

        let api_response = recursive_call_api(&api_key, "https://live.trading212.com/api/v0/equity/history/orders", &cursor, ResponseType::Orders).await;
        // println!("{:?}", api_response);

        (cursor, orders) = match api_response {                    // process_items returns a tuple so we catch both cursor
            Ok(CallResponse::Orders(items)) => process_items(items),            // and orders in this match
            _ => {

                // eprintln!("{}", e);               // doesn't assign tuple but breaks loop so compiler doesn't care
                break
            }
        };

        data.append(&mut orders);
        
    };

    
    
    for item in &mut data {
        item.dateModified = item.dateModified.chars().take(10).collect();    // convert date to daily
    }

    Ok(data)
}



// defining structs for json output to be deserialized into (within recursive_api_call)
#[derive(Debug, Deserialize)]
pub struct Items {
    items: Vec<Order>,

}

#[derive(Debug, Deserialize, Clone)]
pub struct Order {                                            // both the struct and fields have to be public to be accessed in main
    pub id: u64,
    pub ticker: String,
    pub dateModified: String,

    #[serde(default, deserialize_with = "deserialize_null_fields")]    // custom deserialize routine to fill occasional nulls.
    pub filledQuantity: f64,                                  // happens because .json has implementation for null,
                                                              
    #[serde(default, deserialize_with = "deserialize_null_fields")]    // but rust doesn't (and doesn't even treat it as a missing field)    <-\\
    pub fillPrice: f64,

    #[serde(default, deserialize_with = "deserialize_null_fields")]    
    pub filledValue: f64,

    #[serde(default)]    
    pub taxes: Vec<Fee>,

    pub status: String

}

#[derive(Debug, Deserialize)]
pub struct Dividends {
    pub items: Vec<Dividend>,
    pub nextPagePath: Option<String>
    
}

#[derive(Debug, Deserialize)]
pub struct Dividend {
    pub ticker: String,
    pub amount: f64,
    pub paidOn: String
}

// enum to hold the other struct types
#[derive(Debug)]
pub enum CallResponse {
    Orders(Items), // orders
    Divis(Dividends)
}

// the decider for which struct recursive_api_call should return
pub enum ResponseType {
    Orders,
    Divis
}

#[derive(Debug, Deserialize, Clone)]
pub struct Fee {
    pub name: String,
    pub quantity: f32
}

fn deserialize_null_fields<'de, D>(deserializer: D) -> Result<f64, D::Error> where D: Deserializer<'de> {    // the routine itself  <-||
    Option::<f64>::deserialize(deserializer).map(|opt| opt.unwrap_or(0.0))
}



// returns a CallResponse which can be either an Orders or a Dividends variant
pub async fn recursive_call_api(api_key: &str, api_url: &str, current_cursor: &String, response_type: ResponseType) -> Result<CallResponse, Box<dyn Error>>{
 

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

    let status = response.status();
    // let bytes = response.bytes().await?;
    // println!("Raw response: {}", String::from_utf8_lossy(&bytes));

    if status.is_success() {
        match response_type {
            ResponseType::Orders => {let catcher: Items = response.json().await?;
            return Ok(CallResponse::Orders(catcher))},
            ResponseType::Divis => {let catcher: Dividends = response.json().await?;
            return Ok(CallResponse::Divis(catcher))},

        }

    } else {
        if status.as_str().contains("429"){  // 429 means too many requests
            countdown(60);
            let d2_response = Box::pin(recursive_call_api(&api_key, api_url, current_cursor, response_type)).await;  // Box::pin because Rust doesn't allow recursive async funcs that are not boxed
            return d2_response
        } else {
            Err(format!("API call failed: {}", status).into())
        }
    }
}




fn process_items(orders: Items) -> (String, Vec<Order>) {
                                                //vvv if none then none, if some then use in this closure  
    let timestamp = match orders.items.last().and_then(|order| extract_unix(&order.dateModified)) {
        Some(v) => v,                       // if it worked, return unix timestamp as cursor 
        None => String::from("complete")    // it it didn't, return "complete" as cursor 
    };
    eprintln!("processed page: {:?}", timestamp);
    (timestamp, orders.items)
}



pub fn extract_unix(timestamp: &String) -> Option<String> {
    // shadowing
    let timestamp = timestamp.as_str();

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
