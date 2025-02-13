use chrono::NaiveDate;
use std::collections::HashMap;

pub fn calculate_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>

    ) -> Option<HashMap<NaiveDate, f64>> {

    let mut return_history: HashMap<NaiveDate, f64> = HashMap::new();
    let mut portfolio = portfolio_history[0].1.clone();
    // println!("{:?}", portfolio_history);
    let mut outer_holder_value: f64 = 0.0;
    let mut outer_holder_sum: f64 = 0.0;

    for tupleobject in portfolio_history {

        let mut value_total: f64 = 0.0;
        let mut sum_of_mid_returns: f64 = 0.0;

        let date: NaiveDate = tupleobject.0;

        if tupleobject.1.is_empty() == false{
            portfolio = tupleobject.1;
        }else{
          // pass, portfolio remains same as 1 day (iteration) before  
        }

        // if found, value is value and mid return is mid return
        
        for (ticker, (q_0, p_0)) in &portfolio {
            let single_history = complete_prices.get(ticker)?;
            // println!("{:?}", single_history);
            if let Some(v) = single_history.get(&date) {        // if price_history doesn't contain this date, it was a weekend.
                let p_1: f64 = *v;
                let mid_return: f64 = p_0*q_0*(p_1/p_0 - 1.0);
                value_total += q_0*p_0;
                sum_of_mid_returns += mid_return;
                outer_holder_value = value_total;
                outer_holder_sum = sum_of_mid_returns;
            } else {value_total = outer_holder_value.clone();
                    sum_of_mid_returns = outer_holder_sum.clone();
                    };

        };

        let daily_return = (100.0/value_total)*sum_of_mid_returns;
        println!("{:?}, {:?}", date, daily_return);
        return_history.insert(date, daily_return);    
    };
    Some(return_history)
} // if could get it, do as normal. if couldn't get it, last line + break
