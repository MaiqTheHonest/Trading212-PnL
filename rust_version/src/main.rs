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
use std::io::stdin;
use std::process::Command;


fn main() {

    // switch to UTF-8 support by default
    if cfg!(target_os = "windows") {
        let _ = Command::new("chcp").arg("65001").status();
    }

    let euro_borsen = vec![".AS", ".DE", ".MC", ".PA", ".SW", ".MI", ".LS", ".AT", ".BE"];

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
            Ok(v) => {println!("\nOrder import from Trading212: complete");
            v
        },
        Err(e) => panic!("Order import from t212 failed with error code: {}", e)
    };
    
    // REVERSE IS IMPORTANT, as transactions arrive in inverse order
    // after this reverse(), time is aligned with vector index (ascending)
    data.reverse();
    
    // duplicates occur from T212 treating partially filled orders as fully filled
    // so we just remove them. this introduces miniscule price incorrection
    remove_duplicates(&mut data);    
    
    
    // initialize the whole time period
    let time_range = get_time_range(&data).expect("Failed to get time range: ");
    
    let start_date = *time_range.first().unwrap();
    let end_date = *time_range.last().unwrap();





    // getting the fx rate history
    let fx_list: Vec<String> = vec![
        String::from("GBPUSD"), 
        String::from("GBPEUR"), 
        String::from("GBPCAD")];

    let mut fx_history: HashMap<String, HashMap<NaiveDate, f64>> = HashMap::new();

    for fx in fx_list {
        let temp_history: HashMap<NaiveDate, f64> = match yahoo::get_prices(format!("{}=X", fx).as_str(), start_date - Duration::days(2), end_date) {
            Ok(res) => res,
            Err(e) => panic!("FX import from yahoo failed: {e}")
        };
        fx_history.insert(fx, temp_history);

    }
    // yahoo returns no prices for weekends, so I interpolate using Friday's fx rate
    stats::interpolate_weekends(&mut fx_history);




    
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
    // let gbpusd: &f64 = fx_history.get("GBPUSD").unwrap().get(&end_date).unwrap();
    let total_dividends: f64 = dividends::get_dividends()
    .expect("could not fetch dividends");



    for order in &mut data {

        let matcher_date = NaiveDate::from_str(&order.dateCreated).expect("couldn't parse dateCreated: invalid date format");

        // zero filledQuantity means it was a "value" order e.g. "buy £100 of AAPL" instead of "buy 0.5 AAPL at £200"
        // so we need to translate value into quantities. "l_EQ" means a transaction on LSE so it is quoted in pennies
        // and we multiply by 100

        if order.filledQuantity == 0.0 {
            if order.ticker.contains("l_EQ"){
            order.filledQuantity = order.filledValue / (order.fillPrice * 100.0)
            } else {
                order.filledQuantity = order.filledValue / order.fillPrice
            }
        } else {
            // pass
        };

        // changing tickers from T212's format to Yahoo's format
        order.ticker = convert_to_yahoo_ticker(
            order.ticker.clone(), 
            pre_dict_tickers.clone(), 
            post_dict_tickers.clone());

        // multiplying fill prices by respective fx rate
        fx_adjust(&order.ticker, matcher_date, &mut order.fillPrice, &fx_history, &euro_borsen);


        // filtering out cancelled or rejected orders
        if order.status == String::from("FILLED") {
            process_order(&mut portfolio_t, &order, &mut ticker_history, *time_range.last().unwrap());

        } else {
            // pass
        };


        // set portoflio history's element to a correct pair of {Date: portfolio_t}
        let index = time_range.iter().position(|&r| r == matcher_date).expect("time range has no such date");
        portfolio_history[index] = (matcher_date, portfolio_t.clone());
        
    };


    

    let mut complete_prices: HashMap<String, HashMap<NaiveDate, f64>> = HashMap::new();

    println!("\n ticker               lifetime:");
    
    for (ticker, (date1, date2)) in ticker_history.into_iter()  {
        
        println!("{:?},from {:?} to {:?}", ticker, date1, date2);
        let mut single_ticker_history = match yahoo::get_prices(&ticker, date1, date2) {
            Ok(res) => res,
            Err(e) => panic!("Import from yahoo failed with error code: {}", e)
        };


        // multiplying yahoo prices by respective fx rate
        for (date, price) in single_ticker_history.iter_mut() {
            fx_adjust(&ticker, *date, price, &fx_history, &euro_borsen);
        }
        
        complete_prices.insert(ticker, single_ticker_history); 
    };

    // fill in missing weekend prices using Friday prices
    stats::interpolate_weekends(&mut complete_prices);



    
    let return_history = match stats::calculate_returns(portfolio_history, complete_prices, total_dividends) {
        Some(v) => v,
        None => panic!("Calculating returns failed, check dividends arrived")
    };
    



    let naivetime_held = end_date - start_date;
    let days_held: f32 = naivetime_held.num_days() as f32;
    let years_held: f32 = (&days_held)/365.0;
    let months_held: i32 = ((&years_held*12.0) as i32) % 12;                                                                              // vvv this is incorrect
    println!("\n \n Found portfolio of {:.} years, {:.} months, and {:.} days.\n", years_held.floor(), months_held, days_held as i32 % 365 - 30*months_held);
    println!("Current GBP/USD = {:?}", fx_history.get("GBPUSD").unwrap().iter().last());
    println!("Current GBP/EUR = {:?}", fx_history.get("GBPEUR").unwrap().iter().last());



    plotter::display_to_console(&return_history, start_date, end_date);



    // shadowing
    let return_history: Vec<(NaiveDate, f32)> = stats::hashmap_to_sorted_vec(return_history);
    let just_returns: Vec<f32> = stats::strip_dates(return_history);
    
    
    
    let current_return = &just_returns.last().unwrap();
    let annual_return = ((*current_return/100.0 + 1.0).powf(1.0/(&years_held)) - 1.0) * 100.0;
    let daily_returns: Vec<f32> = stats::get_daily_returns(just_returns.clone());
    let (mean, sd, sharpe) = stats::mean_sd_sharpe(&daily_returns);
    
    println!("                    _________________________________________");
    println!("                   |                       |                 |");
    println!("                   | {0: <21} | {1: <15.4} | ", "unrealised PnL(%)", current_return);
    println!("                   |                       |                 |");
    println!("                   | {0: <21} | {1: <15.4} | ", "APR(%)", annual_return);
    println!("                   |                       |                 |");
    println!("                   | {0: <21} | {1: <15.4} | ", "std. deviation", sd);
    println!("                   |                       |                 |");
    println!("                   | {0: <21} | {1: <15.4} | ", "Sharpe ratio", sharpe);
    println!("                   |                       |                 |");
    println!("                   | {0: <21} | {1: <15.4} | ", "daily avg. return(%)", mean);
    println!("                   |                       |                 |");
    println!("                    ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾ \n \n");

    println!("Press any key to exit...");
    stdin().read_line(&mut String::new()).unwrap();
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


//fx adj should be here
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

            let y_borse = match pre_dict_tickers.get(&*borse) {    // the most deranged deref usage I've done
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
    




// finds the correct currency using ticker name and borse list, then fetches it from fx_history and multiplies by it
fn fx_adjust(ticker: &String, matcher_date: NaiveDate, price: &mut f64, fx_history: &HashMap<String, HashMap<NaiveDate, f64>>, euro_borsen: &Vec<&str>) {
                
    let contains_any: bool = euro_borsen.iter().any(|&b| ticker.contains(b));

    if ticker.contains(".TO") {
        let temp_fx = fx_history
            .get("GBPCAD")
            .unwrap()
            .get(&matcher_date)
            .expect(&format!("couldn't get FX GBPCAD for {}", &matcher_date));
        *price = *price / temp_fx;
    } else {
        if contains_any {
            let temp_fx = fx_history
                .get("GBPEUR")
                .unwrap()
                .get(&matcher_date)
                .expect(&format!("couldn't get FX GBPEUR for {}", &matcher_date));
            *price = *price / temp_fx;
        } else if ticker.contains(".L") {
            *price = *price / 100.0
            // do nothing as it is already GBP and not other currency or GBX;
        } else {
            let temp_fx = fx_history
                .get("GBPUSD")
                .unwrap()
                .get(&matcher_date)
                .expect(&format!("couldn't get FX GBPUSD for {}", &matcher_date));
            *price = *price / temp_fx;
        }
    };
}