use chrono::{Days, NaiveDate};
use std::collections::HashMap;


const RISK_FREE_RATE: f32 = 0.03;
const N_MARKET_DAYS: f32 = 252.0;



// unrealized, non-TWR, non-MWR
pub fn calc_unreal_returns(
    portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)>,
    complete_prices: HashMap<String, HashMap<NaiveDate, f64>>,
    total_dividends: f64

    ) -> Option<(HashMap<NaiveDate, f64>, HashMap<NaiveDate, (f64, f64)>)> {

    let mut return_history: HashMap<NaiveDate, f64> = HashMap::new();
    let mut portfolio = portfolio_history[0].1.clone();
    let mut cb_mv_history: HashMap<NaiveDate, (f64, f64)> = HashMap::new();

    let mut volume_total: f64 = 0.0;
    let mut volume_covered: f64 = 0.0;


    // this first loop is to calculate total volume
    for tupleobject in &portfolio_history {

        if tupleobject.1.is_empty() == false{
            portfolio = tupleobject.clone().1;
        }else{}
        for (ticker, (q_0, p_0)) in &portfolio {
            volume_total += q_0*p_0;
        };

    };


    // this second loop calculates daily returns by weighting dividends against total volume
    for tupleobject in portfolio_history {

        let mut sum_of_abs_returns: f64 = 0.0;

        let date: NaiveDate = tupleobject.0;

        if tupleobject.1.is_empty() == false{
            portfolio = tupleobject.1;
        }else{
          // pass, portfolio remains same as 1 day (iteration) before  
        };


        let mut market_val: f64 = 0.0;
        let mut cost_basis: f64 = 0.0;

        for (ticker, (q_0, p_0)) in &portfolio {

            let single_history = complete_prices.get(ticker)?; // get price history for ticker


            if let Some(v) = single_history.get(&date) {      // get specific day from that price history
                let p_1: f64 = *v;

                market_val += p_1*q_0;
                cost_basis += q_0*p_0;
                
                let abs_return: f64 = p_0*q_0*(p_1/p_0 - 1.0);
                sum_of_abs_returns += abs_return;

            } else {
                //println!("ERROR: historical price not found for ticker {:?} on date {:?}", ticker, date)
                };
        };

        volume_covered += cost_basis;

        let daily_return = (100.0/cost_basis)*(sum_of_abs_returns + total_dividends*(volume_covered/volume_total));

        return_history.insert(date, daily_return);
        cb_mv_history.insert(date, (cost_basis, market_val));    
    };
    
    Some((return_history, cb_mv_history))
} 





pub fn hashmap_to_sorted_vec<T>(hashmap: HashMap<NaiveDate, T>) -> Vec<(NaiveDate, T)> {
    
    let mut vec: Vec<(NaiveDate, T)> = hashmap.into_iter().collect();
    
    vec.sort_by_key(|(date, _)| *date);               // sort by naivedate
    
    vec
}





pub fn strip_dates(return_history: Vec<(NaiveDate, f32)>) -> Vec<f32> {

    let (_, just_returns): (Vec<NaiveDate>, Vec<f32>) = return_history.into_iter().unzip();

    just_returns
}





pub fn interpolate<T: Clone>(history: &mut Vec<(NaiveDate, T)>) {
    let mut i = 0;

    while i + 1 < history.len() {
        let current_date = history[i].0; 
        let next_date = current_date.checked_add_days(Days::new(1)).unwrap();

        if next_date != history[i + 1].0 {
            let cloned_value = history[i].1.clone();
            history.insert(i + 1, (next_date, cloned_value));
        } else {
            i += 1;
        }
    }
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




pub fn interpolate_weekends(full_history: & mut HashMap<String, HashMap<NaiveDate, f64>>){
        
    for _ in 0..5{
        for (_, single_element) in full_history.iter_mut() {
            for (key, value) in single_element.clone() {
                if let Some(next_day) = key.checked_add_days(Days::new(1)) {
                    if !single_element.contains_key(&next_day) {
                        single_element.insert(next_day, value);
                    }
                }
            }
        }
    }
}





// finds the correct currency using ticker name and borse list, then fetches it from fx_history and multiplies by it
pub fn fx_adjust(ticker: &String, matcher_date: NaiveDate, price: &mut f64, fx_history: &HashMap<String, HashMap<NaiveDate, f64>>) {
    
    let euro_borsen = vec![".AS", ".DE", ".MC", ".PA", ".SW", ".MI", ".LS", ".AT", ".BE"]; 

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


pub fn calculate_benchmark_returns(bench_returns: Vec<(NaiveDate, f64)>) -> Vec<f32>{
    let mut returns = Vec::new();

    for i in 1..bench_returns.len() {
        let price_today = bench_returns[i].1;
        let initial_price = bench_returns[0].1;

        let return_today = ((price_today / initial_price) - 1.0) * 100.0;
        
        returns.push(return_today as f32);
    }
    returns
}





pub fn covariance(just_returns: &Vec<f32>, bench_returns: &Vec<f32>, mean: f32, mean_bench: f32) -> f32 {
  
    let n = just_returns.len() as f32;

    just_returns.iter()
        .zip(bench_returns.iter())
        .map(|(r_p, r_b)| (r_p - mean) * (r_b - mean_bench))
        .sum::<f32>() / n
}






// Newton-Raphson method for money-weighted rate of return

pub fn mwrr(cashflows: &Vec<(NaiveDate, f64)>, guess: f64) -> Option<f64> {

    const ITERS: usize = 1000;
    const TOLERANCE: f64 = 0.01;

    //  || cashflows.len() == 1 
    if cashflows.is_empty(){
        return None;
    };

    let t0 = cashflows[0].0;
    let total_days = (cashflows.last().unwrap().0 - cashflows.first().unwrap().0).num_days() as f64;
    let npv = |rate: f64| -> f64 {
        cashflows.iter().map(|cf| {
            let days = (cf.0 - t0).num_days() as f64;
            cf.1 / (1.0 + rate).powf(days / total_days)
        }).sum()
    };

    // the derivative of the cash flow sum function to be used in x1 = x0 - f(x0)/f'(x0)
    let npv_derivative = |rate: f64| -> f64 {
        cashflows.iter().map(|cf| {
            let days = (cf.0 - t0).num_days() as f64;
            let exp = days / total_days;
            -cf.1 * exp / (1.0 + rate).powf(exp + 1.0)
        }).sum()
    };

    let try_converge = |guess: f64| -> Option<f64> {
        let mut rate = guess;
        for _ in 0..ITERS {
            let f = npv(rate);
            let f_dash = npv_derivative(rate);

            // check for non-zero derivative or just a really small number that leads to large step size 
            if f_dash.abs() < 1e-10 {break}

            let next_rate = rate - f / f_dash;

            if (next_rate - rate).abs() < TOLERANCE {
                return Some(next_rate);
            }
            rate = next_rate;
        }
        None
    };

    match try_converge(guess) {
        Some(rate) => Some(rate),        // cash flow function might not converge if guess is +ve and irr is -ve
        None => try_converge(-guess)    // or vice versa so we try -guess if it didnt work first time around
    }
}