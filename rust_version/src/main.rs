use std::collections::HashMap;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::error::Error;
use std::fs;


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    call_api().await?;
    Ok(())
}


async fn call_api() -> Result<(), Box<dyn Error>> {

    let api_key = fs::read_to_string("api_key.txt")
    .expect("could not find api_key.txt");

    let api_url = "https://live.trading212.com/api/v0/equity/history/orders";

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(&api_key)?);

    let params = HashMap::from([
        ("cursor", ""),
        ("ticker", "SQM_US_EQ"),
        ("limit", "10")]);


    let client = reqwest::Client::new();
    let response = client
        .get(api_url)
        .headers(headers)
        .query(&params)
        .send()
        .await?;

    if response.status().is_success() {
        let portfolio_data: Value = response.json().await?;    //Value is a serde_json struct to store response
        

        // shadowing to convert to vector
        let portfolio_data = &portfolio_data["items"].as_array().unwrap();

        println!("{:?}", &portfolio_data.last());

        // println!("last iso {:?}", &portfolio_data["items"])
    } else {
        eprintln!("failed to fetch or authorize: {}", response.status());
    }
    Ok(())

}

