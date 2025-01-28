use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde_json::Value;
use std::error::Error;
use std::fs;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let contents = fs::read_to_string("api_key.txt")
    .expect("could not find api_key.txt");
    let api_url = "https://live.trading212.com/api/v0/equity/portfolio";

    let api_token= &contents;

    let mut headers = HeaderMap::new();
    headers.insert(AUTHORIZATION, HeaderValue::from_str(api_token)?);

    let client = reqwest::Client::new();
    let response = client
        .get(api_url)
        .headers(headers)
        .send()
        .await?;

    if response.status().is_success() {
        let portfolio_data: Value = response.json().await?;    //Value is a serde_json struct to store response
        println!("{:#}", portfolio_data);
        
        println!("gottenvalue = {}", &portfolio_data[0]["averagePrice"]);
    } else {
        eprintln!("failed to fetch or authorize: {}", response.status());
    }
    
    Ok(())
}

