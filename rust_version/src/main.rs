mod t212;
use chrono::{NaiveDate, Duration};
// use serde::de::Error;
use std::{collections::{hash_map::Entry, HashMap}, error::Error, str::FromStr};
use std::collections::HashSet;
use crate::t212::Order;

fn main() {
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



    for order in &mut data {

        // dealing with edge cases: l_EQ means LSE transaction, which is quoted in pennies
        // so we multiply by 100. Also where value transaction, we translate into quantities
        if order.filledQuantity == 0.0 && order.ticker.contains("l_EQ") {
            order.filledQuantity = (order.filledValue * 100.0) / order.fillPrice    
        } else {
            // pass
        };

        // filtering out cancelled or rejected orders
        if order.status == String::from("FILLED") {
            process_order(&mut portfolio_t, &order, &mut ticker_history, *time_range.last().unwrap());

        } else {
            // pass
        };

        
        // set portoflio history's element to a correct pair of Date: portfolio_t
        let matcher = NaiveDate::from_str(&order.dateCreated).expect("invalid date format");

        let index = time_range.iter().position(|&r| r == matcher).expect("time range has no such date");
        portfolio_history[index] = (matcher, portfolio_t.clone());
        
    }

    // let blarg = NaiveDate::from_str("2025-02-03").unwrap();
    println!("{:?}", ticker_history);
    
}



fn remove_duplicates(orders: &mut Vec<Order>) {
    let mut seen = HashSet::new();
    orders.retain(|order| seen.insert(order.id));
}



fn get_time_range(data: &Vec<Order>) -> Result<Vec<NaiveDate>, Box<dyn Error>> {

    let root_date = data.first().ok_or("couldn't get last order")?.dateCreated.as_str();    

    // ^^^ last() returns an option, ok_or converts it to result, "?" propagates the error

    let mut start_date = NaiveDate::parse_from_str(&root_date, "%Y-%m-%d")?;

    let term_date = data.last().ok_or("couldn't get first order")?.dateCreated.as_str();   
    let end_date = NaiveDate::parse_from_str(&term_date, "%Y-%m-%d")?;

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

    // println!("{}", order.dateCreated);
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
                    ticker_history.insert(ticker.clone(), (date, last_date));
                    } else {
                        *q_0 += q_1;                                           // if sold some (not everything)
                        ticker_history.insert(ticker, (date, last_date));
                    };
        };
    },
        Entry::Vacant(vacant) => {
            vacant.insert((q_1, p_1));
            ticker_history.insert(ticker.clone(), (date, last_date));
        },
    };
}                        // returns nothing, just amends portfolio in-place

