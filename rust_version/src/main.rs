mod t212;
use chrono::{NaiveDate, Duration};
use crate::t212::Order;

fn main() {
    let data = match t212::get_orders() {
        Ok(v) => {println!("Import from t212 successful");
        v
    },
        Err(e) => panic!("Import from t212 failed with error code: {}", e)
    };
    



    // let end_date = NaiveDate::from_ymd_opt(2024, 2, 7).unwrap();
    // println!("{:?}", start_date.iter_days().take(3))
    let aff = get_time_range(data).unwrap();
    println!("{:?}" , aff)
    // data is Vec<Order>
}

fn get_time_range(data: Vec<Order>) -> Option<Vec<NaiveDate>> {

    // last() is actually the first trasaction
    let root_date = data.last()?.dateCreated.as_str();    

    // this will panic! if couldn't parse so no need to propagate error further
    let mut start_date = NaiveDate::parse_from_str(&root_date, "%Y-%m-%d").expect("couldn't parse NaiveDate");

    let term_date = data.first()?.dateCreated.as_str();   
    let end_date = NaiveDate::parse_from_str(&term_date, "%Y-%m-%d").expect("couldn't parse NaiveDate");

    let mut time_range = Vec::new();

    while start_date < end_date {
        time_range.push(start_date);
        start_date += Duration::days(1);
    }

    Some(time_range)    // return
}

