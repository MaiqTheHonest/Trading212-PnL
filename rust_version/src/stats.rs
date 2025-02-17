use chrono::NaiveDate;
use std::collections::HashMap;


const RISK_FREE_RATE: f32 = 0.03;
const N_MARKET_DAYS: f32 = 252.0;

pub fn calculate_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>,
    total_dividends: f64

    ) -> Option<HashMap<NaiveDate, f64>> {

    let mut return_history: HashMap<NaiveDate, f64> = HashMap::new();
    let mut portfolio = portfolio_history[0].1.clone();
    // println!("{:?}", portfolio_history);

    let mut volume_total: f64 = 0.0;
    let mut volume_covered: f64 = 0.0;
    let mut outer_holder_value: f64 = 0.0;
    let mut outer_holder_sum: f64 = 0.0;


    // this first loop is to calculate total volume
    for tupleobject in &portfolio_history {

        if tupleobject.1.is_empty() == false{
            portfolio = tupleobject.clone().1;
        }else{}
        for (_, (q_0, p_0)) in &portfolio {
            volume_total += q_0*p_0;
        };

    };


    // this second loop calculates daily returns by weighting dividends against total volume
    for tupleobject in portfolio_history {

        let mut value_total: f64 = 0.0;
        let mut sum_of_mid_returns: f64 = 0.0;

        let date: NaiveDate = tupleobject.0;

        if tupleobject.1.is_empty() == false{
            portfolio = tupleobject.1;
        }else{
          // pass, portfolio remains same as 1 day (iteration) before  
        };

        // if found, value is value and mid return is mid return
        
        for (ticker, (q_0, p_0)) in &portfolio {

            let single_history = complete_prices.get(ticker)?;

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

        volume_covered += value_total;

        let daily_return = (100.0/value_total)*(sum_of_mid_returns + total_dividends*(volume_covered/volume_total));

        // println!("{:?}, {:?}", date, daily_return);
        return_history.insert(date, daily_return);    
    };
    

    Some(return_history)
} 




pub fn hashmap_to_sorted_vec(hashmap: HashMap<NaiveDate, f64>) -> Vec<(NaiveDate, f32)> {
    let mut vec: Vec<(NaiveDate, f32)> = hashmap.into_iter()
        .map(|(date, value)| (date, value as f32))    // mind the conversion
        .collect();
    
    vec.sort_by_key(|(date, _)| *date);               // sort by naivedate
    
    vec
}



pub fn strip_dates(return_history: Vec<(NaiveDate, f32)>) -> Vec<f32> {

    let (_, just_returns): (Vec<NaiveDate>, Vec<f32>) = return_history.into_iter().unzip();

    just_returns
}


pub fn get_daily_returns(mut just_returns: Vec<f32>) -> Vec<f32> {

    let mut prev_value: f32 = 100.0;

    for value in just_returns.iter_mut() {
        let daily_return = ((*value + 100.0 - prev_value) / prev_value) * 100.0;
        *value = daily_return;  
        prev_value = *value + prev_value;  

    };
    just_returns

}


pub fn mean_sd_sharpe(just_returns: &Vec<f32>) -> (f32, f32, f32){
    let len = just_returns.len() as f32;
    let blarg: f32 = just_returns.iter().map(|value| (value/100.0 + 1.0)).product::<f32>();
    let mean: f32 = (blarg.powf(1.0 / len) - 1.0)*100.0;
    let variance: f32 = just_returns.iter().map(|value| (value - mean).powi(2)).sum::<f32>() / (len - 1.0);
    let daily_risk_free_rate: f32 = ((1.0 + RISK_FREE_RATE).powf(1.0 / N_MARKET_DAYS) - 1.0)*100.0;
    let sharpe: f32 = (mean - daily_risk_free_rate)/(variance.sqrt());

    (mean, variance.sqrt(), sharpe)
}