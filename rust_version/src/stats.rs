use chrono::NaiveDate;
use std::collections::HashMap;

pub fn calculate_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>

    ) -> Option<HashMap<NaiveDate, f64>> {

    let mut return_history: HashMap<NaiveDate, f64> = HashMap::new();
    let mut portfolio = portfolio_history[0].1.clone();
    // println!("{:?}", portfolio_history);
    let mut q_total: f64 = 0.0;
    let mut sum_of_mid_returns: f64 = 0.0;
    for tupleobject in portfolio_history {

        let date: NaiveDate = tupleobject.0;

        if tupleobject.1.is_empty() == false{
            portfolio = tupleobject.1;
        }else{
          // pass, portfolio remains same as 1 day (iteration) before  
        }


        
        for (ticker, (q_0, p_0)) in portfolio.clone().into_iter() {
            let single_history = complete_prices.get(&ticker)?;
            // println!("{:?}", single_history);
            if let Some(v) = single_history.get(&date) {        // if price_history doesn't contain this date, it was a weekend.
                let p_1: f64 = *v;
                let mid_return: f64 = q_0*(p_1/p_0 - 1.0);
                q_total += q_0;
                sum_of_mid_returns += mid_return;
            } else {break};

        };

        let daily_return = (100.0/q_total)*sum_of_mid_returns;
        println!("{:?}", daily_return);
        return_history.insert(date, daily_return);    
    };
    Some(return_history)
}// if could get it, do as normal. if couldn't get it, last line + break
