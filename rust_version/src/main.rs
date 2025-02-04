use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::error::Error;
use std::fs;
use chrono::DateTime;
use serde::Deserialize;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let api_response = call_api().await;
    match api_response {
        Ok(v) => process_items(v),
        Err(e) => eprintln!("{}", e),
    }

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

}


// LLALALALALALALLAA
async fn call_api() -> Result<Items, Box<dyn Error>> {

    let api_key = fs::read_to_string("api_key.txt")
    .expect("could not find api_key.txt");

    let api_url = "https://live.trading212.com/api/v0/equity/history/orders";

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);

    let params = HashMap::from([
        ("cursor", ""),
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
        let catcher: Items = response.json().await?;    //Value is a serde_json struct to store response
        Ok(catcher)
        
        // shadowing to convert to vector
        // let portfolio_data = portfolio_data["items"].as_array().unwrap();    // later add proper err management with match
        // let tickers = &portfolio_data.get("tickers");
        // println!("{:?}", &portfolio_data)

        // let next_cursor = extract_unix(&portfolio_data);    // later add proper err management with match
        // println!("{:?}", portfolio_data);
        // println!("{:?}", next_cursor);
        
        // if let Some(stuff) = next_cursor {
        //     println!("{:?}", stuff)
        // }

        

        // println!("last iso {:?}", &portfolio_data["items"])
    } else {
        // eprintln!("failed to fetch or authorize: {}", response.status());
        Err(format!("API call failed: {}", response.status()).into())
    }
    

}

fn process_items(orders: Items) {

    let mut ay = orders.items.last();

    println!("{:?}", &orders);
    println!("{:?}", &ay)
}


// fn extract_unix(orders: &Vec<Value>) -> Option<String> {
//     let orders = orders
//     .last()?
//     .get("dateCreated")?
//     .as_str()?;

//     let orders = DateTime::parse_from_rfc3339(orders)
//     .ok()?
//     .timestamp_millis()
//     .to_string();

//     Some(orders)
// }
