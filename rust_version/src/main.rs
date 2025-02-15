mod t212;
mod yahoo;
mod stats;
mod dividends; // redundant
mod plotter;
use chrono::{Duration, NaiveDate, Utc};
// use serde::de::Error;
use std::{collections::{hash_map::Entry, HashMap}, error::Error, str::FromStr};
use std::collections::HashSet;
use crate::t212::Order;

fn main() {

    let pre_dict_tickers = HashMap::from([       // exchange codes
        ("a", "AS"),
        ("d", "DE"),
        ("e", "MC"),
        ("p", "PA"),
        ("l", "L"),
        ("s", "SW"),
        ("m", "MI")
        ]);
    
    let post_dict_tickers = HashMap::from([      // country codes
        ("PT", "LS"),
        ("AT", "VI"),
        ("BE", "BR"),
        ("CA", "TO")
        ]);

    
    let mut data = match t212::get_orders() {
        Ok(v) => {println!("Import from t212 successful");
        v
    },
        Err(e) => panic!("Import from t212 failed with error code: {}", e)
    };

    // REVERSE IS IMPORTANT, as transactions arrive in inverse order
    // after this reverse(), time is aligned with vector index (ascending)
    data.reverse();

    // duplicates occur from T212 treating partially filled orders as fully filled
    // so we just remove them. this introduces miniscule price incorrection
    remove_duplicates(&mut data);    


    // initialize the whole time period
    let time_range = get_time_range(&data).expect("Failed to get time range: ");

    // initialize portfolio history based on time_range
    let mut portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)> = time_range.clone()
    .into_iter()
    .map(|d| (d, HashMap::new()))    // create empty portfolio hashmap for every date
    .collect();

    // initialize where we store dates for which certain tickers wiere present in portfolio
    let mut ticker_history: HashMap<String, (NaiveDate, NaiveDate)> = HashMap::new();

    // initialize portfolio "holder/folder" at time t
    let mut portfolio_t: HashMap<String, (f64, f64)> = HashMap::new();

    // get dividends to be passed into return calculation
    let total_dividends: f64 = dividends::get_dividends().expect("could not fetch dividends");
    // let daily_dividend: f64 = total_dividends / time_range.iter().count() as f64;
    // println!("total dividends: {:?}, N of days: {:?}", total_dividends, time_range.iter().count() as f64);


    for order in &mut data {

        // dealing with edge cases: l_EQ means LSE transaction, which is quoted in pennies
        // so we multiply by 100. Also where value transaction, we translate into quantities
        if order.ticker.contains("l_EQ") {
            order.fillPrice = order.fillPrice / 100.0 * 1.20;
        }else{
            // pass
        }
        if order.filledQuantity == 0.0 {
            order.filledQuantity = (order.filledValue * 1.20) / order.fillPrice  

        } else {
            // pass
        };

        // changing tickers from T212's format to Yahoo's format
        order.ticker = convert_to_yahoo_ticker(
            order.ticker.clone(), 
            pre_dict_tickers.clone(), 
            post_dict_tickers.clone());

        // filtering out cancelled or rejected orders
        if order.status == String::from("FILLED") {
            process_order(&mut portfolio_t, &order, &mut ticker_history, *time_range.last().unwrap());

        } else {
            // pass
        };




        // set portoflio history's element to a correct pair of {Date: portfolio_t}
        let matcher = NaiveDate::from_str(&order.dateCreated).expect("invalid date format");

        let index = time_range.iter().position(|&r| r == matcher).expect("time range has no such date");
        portfolio_history[index] = (matcher, portfolio_t.clone());
        
    }


    
    // let mut price_history: HashMap<NaiveDate, f64> = HashMap::new();
    let mut complete_prices: HashMap<String, HashMap<NaiveDate, f64>> = HashMap::new();

    for (ticker, (date1, date2)) in ticker_history.into_iter() {
        println!("{:?},{:?},{:?}", ticker, date1, date2);
        let mut single_ticker_history = match yahoo::get_prices(&ticker, date1, date2) {
            Ok(res) => res,
            Err(e) => panic!("Import from yahoo failed with error code: {}", e)
        };
        if ticker.contains(".L") {
            for (_, val) in single_ticker_history.iter_mut() {
                *val = *val / 100.0 * 1.20;
            };
            
        }else {}
        complete_prices.insert(ticker, single_ticker_history);
    }


    let return_history = match stats::calculate_returns(portfolio_history, complete_prices, total_dividends) {
        Some(v) => v,
        None => panic!("Calculating returns failed, check dividends arrived")
    };
    
    plotter::display_to_console(return_history, *time_range.first().unwrap());
    
}



fn remove_duplicates(orders: &mut Vec<Order>) {
    let mut seen = HashSet::new();
    orders.retain(|order| seen.insert(order.id));
}



fn get_time_range(data: &Vec<Order>) -> Result<Vec<NaiveDate>, Box<dyn Error>> {

    let root_date = data.first().ok_or("couldn't get first order")?.dateCreated.as_str();    

    // ^^^ last() returns an option, ok_or converts it to result, "?" propagates the error

    let mut start_date = NaiveDate::parse_from_str(&root_date, "%Y-%m-%d")?;

    // let term_date = data.last().ok_or("couldn't get last order")?.dateCreated.as_str();   
    // let end_date = NaiveDate::parse_from_str(&term_date, "%Y-%m-%d")?;
    let end_date = Utc::now().date_naive();

    let mut time_range = Vec::new();

    while start_date <= end_date {
        time_range.push(start_date);
        start_date += Duration::days(1);
    }

    Ok(time_range)    // return
}



fn process_order(
    portfolio_t: &mut HashMap<String, (f64, f64)>,
    order: &Order,
    ticker_history: &mut HashMap<String, (NaiveDate, NaiveDate)>,
    last_date: NaiveDate) {

    let q_1 = order.filledQuantity;
    let p_1 = order.fillPrice;
    let date = NaiveDate::from_str(order.dateCreated.as_str()).unwrap();
    let ticker = order.ticker.clone();

    match portfolio_t.entry(order.ticker.clone()) {
        Entry::Occupied(mut occupied) => {
            let (q_0, p_0) = occupied.get_mut();

            if *q_0 + q_1 == 0.0 {                                              // if sold everything
                occupied.remove();    // removes ticker from portfolio

                let (keeps_date, _) = ticker_history.get(&ticker).unwrap();
                ticker_history.insert(ticker, (*keeps_date, date));
            } else {
                if q_1 >= 0.0 {                                                // if bought some
                    *p_0 = (*q_0* *p_0 + q_1*p_1)/(*q_0 + q_1);
                    *q_0 += q_1;

                    ticker_history.entry(ticker.clone())
                    .and_modify(|e| e.1 = last_date.clone())
                    .or_insert((date.clone(), last_date.clone()));
                    } else {
                        *q_0 += q_1;                                           // if sold some (not everything)

                        ticker_history.entry(ticker.clone())
                        .and_modify(|e| e.1 = last_date.clone())
                        .or_insert((date.clone(), last_date.clone()));
                    };
        };
    },
        Entry::Vacant(vacant) => {
            vacant.insert((q_1, p_1));
            ticker_history.insert(ticker.clone(), (date, last_date));
        },
    };
}      // returns nothing, just amends portfolio_t and ticker_history in-place



fn convert_to_yahoo_ticker(
    ticker: String,
    pre_dict_tickers: HashMap<&str, &str>,
    post_dict_tickers:HashMap<&str, &str>
    ) -> String {
    
    let mut returnable_ticker: String = String::new();

    if let Some(pos) = ticker.rfind("_EQ") {
        let before_eq = &ticker[..pos];                              // take what's before _EQ
        let parts: Vec<&str> = before_eq.split('_').collect();       // separate what's left by _ and turn into collection



        if parts.len() == 1 {
            let mut pre = parts[0];
            let borse = pre.chars().last().unwrap().to_string();

            let mut y_borse = match pre_dict_tickers.get(&*borse) {    // the most deranged deref usage I've done
                Some(v) => v,
                None => panic!("couldn't find exchange with postfix: {}", &borse)
            }.to_owned();

            pre = &pre[..pre.len() - 1];
            // let corrupt_tickers = vec!["VUAA"];                   // some Milano tickers don't work so we try same stock in Germany 
            // if corrupt_tickers.contains(&pre) {
            //     y_borse = "L";
            // }
            returnable_ticker = format!("{}.{}", pre, y_borse);



        } else if parts.len() == 2 {
            let borse = parts[1];
            if borse == "US" {return parts[0].to_string()}         // if postfix is "US", then no postfix to yahoo ticker is needed


            let y_borse = match post_dict_tickers.get(&*borse) {    
                Some(v) => v,
                None => panic!("couldn't find exchange with postfix: {}", &borse)
            };
            returnable_ticker = format!("{}.{}", parts[0], y_borse);
        } else {}
    };

    returnable_ticker
}
    


