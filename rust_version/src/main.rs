mod t212;
use chrono::{NaiveDate, Duration};
// use serde::de::Error;
use std::{collections::{HashMap, hash_map::Entry}, error::Error};
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

    println!("{:?}", data);

    let time_range = get_time_range(&data).expect("Failed to get time range: ");
    // println!("{:?}", time_range);

    let mut portfolio_t: HashMap<String, (f64, f64)> = HashMap::new();

    for order in data {

        if order.status == String::from("FILLED") {
            process_order(&mut portfolio_t, &order);
        } else {
            // pass
        }
            
    }

    println!("{:?}", portfolio_t)
}



fn remove_duplicates(orders: &mut Vec<Order>) {
    let mut seen = HashSet::new();
    orders.retain(|order| seen.insert(order.id));
}



fn get_time_range(data: &Vec<Order>) -> Result<Vec<NaiveDate>, Box<dyn Error>> {


    let root_date = data.first().ok_or("couldn't get last order")?.dateCreated.as_str();    

    // last() returns an option. ok_or converts it to result. "?" propagates error

    let mut start_date = NaiveDate::parse_from_str(&root_date, "%Y-%m-%d")?;

    let term_date = data.last().ok_or("couldn't get first order")?.dateCreated.as_str();   
    let end_date = NaiveDate::parse_from_str(&term_date, "%Y-%m-%d")?;

    let mut time_range = Vec::new();

    while start_date < end_date {
        time_range.push(start_date);
        start_date += Duration::days(1);
    }

    Ok(time_range)    // return
}



// fn get_sparse_portfolio 

//if order.status == String::from("FILLED") 
            
// add order to port_t (change port_t)
// add port_t to port_history


fn process_order(portfolio_t: &mut HashMap<String, (f64, f64)>, order: &Order) {

    // println!("{}", order.dateCreated);

    let q_1 = order.filledQuantity;
    let p_1 = order.fillPrice;

    match portfolio_t.entry(order.ticker.clone()) {
        Entry::Occupied(mut occupied) => {
            let (q_0, p_0) = occupied.get_mut();

            if *q_0 + q_1 == 0.0 {
                occupied.remove();
            } else {
                if q_1 >= 0.0 {
                    *p_0 = (*q_0* *p_0 + q_1*p_1)/(*q_0 + q_1);    // try using assert eq instead
                    *q_0 += q_1;
                    } else {
                        *q_0 += q_1;
                    };
        };
    },
        Entry::Vacant(vacant) => {
            vacant.insert((q_1, p_1));
        },
    };
}                        // returns nothing, just amends portfolio in-place
