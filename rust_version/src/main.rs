mod t212;
mod yahoo;
mod stats;
mod dividends;
mod plotter;
use rgb::RGB8;
use chrono::{Datelike, Days, Duration, NaiveDate, Utc};
use std::{collections::{hash_map::Entry, BTreeMap, HashMap}, default, error::Error, fs::File, process, str::FromStr};
use std::collections::HashSet;
use crate::{stats::{hashmap_to_btree, hashmap_to_sorted_vec, interpolate, mwrr}, t212::{Dividend, Order}};
use std::io::{self, Write, BufReader};
use std::process::Command;
use std::fs::{OpenOptions, read_to_string};
use plotter::*;
use serde_json::{from_reader, to_writer};

fn main() {

    // READING JSON WITH CUSTOM TICKERS #########################
    let path = "custom_tickers.json";
    let file = OpenOptions::new()
    .read(true)
    .write(true)
    .create(true)
    .open(path).expect("json reader fail");

    let mut custom_tickers: HashMap<String, String> = match from_reader(&file) {
        Ok(v) => v,
        Err(_) => HashMap::new()
    };
    // ########################################################





    // GETTING ORDERS AND ACTIVE TIME RANGE ###################
    let api_key: String = read_to_string("api_key.txt").expect("could not find api_key.txt").split_whitespace().collect();

    let mut data = match t212::get_orders(&api_key) {
        Ok(v) => {
            if v.is_empty(){
                
                eprintln!("Error: invalid API key or new account with 0 orders");
                process::exit(1)
            } else {
                println!("\nOrder import from Trading212: complete");
                println!("fetched a total of {} orders \n ", v.len());
                v
            }
        },
        Err(e) => {
            println!("Order import from t212 failed with error code: {}", e);
            process::exit(1)
        }
};
    // REVERSE IS IMPORTANT, as transactions arrive in inverse order
    // after this reverse(), time is aligned with vector index (ascending)
    data.reverse();
    // println!("{:#?}", data.get(0..100));
    // duplicates occur from T212 treating partially filled orders as fully filled
    // so we just remove them. this introduces price incorrection but partial fills are rare at T212
    remove_duplicates(&mut data);    
    
    
    // initialize the whole time period
    let time_range = get_time_range(&data).expect("Failed to get time range: ");
    
    let start_date = *time_range.first().unwrap();
    let end_date = *time_range.last().unwrap();
    //#########################################################





    // GETTING FX RATES #######################################
    let fx_list: Vec<String> = vec![
        String::from("GBPUSD"), 
        String::from("GBPEUR"), 
        String::from("GBPCAD")];

    let mut fx_history: HashMap<String, HashMap<NaiveDate, f64>> = HashMap::new();

    for fx in fx_list {
        let temp_history: HashMap<NaiveDate, f64> = match yahoo::get_prices(format!("{}=X", fx).as_str(), start_date - Duration::days(2), end_date, &mut custom_tickers) {
            Ok(res) => res,
            Err(e) => panic!("FX import from yahoo failed: {e}")
        };
        fx_history.insert(fx, temp_history);

    }
    // yahoo returns no prices for weekends, so I interpolate using Friday's fx rate
    stats::interpolate_weekends(&mut fx_history);
    //##########################################################



    

    // INITIALIZING PORTFOLIO AND RETURN VARIABLES #############
    let mut portfolio_history: Vec<(NaiveDate, HashMap<String, (f64, f64)>)> = time_range.clone()
    .into_iter()
    .map(|d| (d, HashMap::new()))    // create empty portfolio hashmap for every date
    .collect();

    // initialize where we store cash flows (only for use in mwrr calculations)
    let mut cash_flows: HashMap<NaiveDate, f64> = HashMap::new();

    // initialize where we store dates for which certain tickers wiere present in portfolio
    let mut ticker_history: HashMap<String, (NaiveDate, NaiveDate)> = HashMap::new();

    // initialize portfolio "holder/folder" at time t
    let mut portfolio_t: HashMap<String, (f64, f64)> = HashMap::new();
    
    // initialize where we store realized returns
    let mut real_returns: HashMap<NaiveDate, (f64, f64)> = HashMap::new();

    // initialize stock prices
    let mut complete_prices: HashMap<String, HashMap<NaiveDate, f64>> = HashMap::new();

    // get dividends to be passed into return calculation
    let mut dividend_history = dividends::get_dividends(&api_key).expect("could not fetch dividends");
    dividend_history.reverse();

    // initialize storage of each ticker's total dividends
    let mut dividend_library: HashMap<String, f64> = HashMap::new();
    // #########################################################





    // PARSING, FILTERING AND FORMATTING ORDERS ################
    for order in &mut data {

        let matcher_date = NaiveDate::from_str(&order.dateModified).expect("couldn't parse dateModified: invalid date format");

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
        order.ticker = yahoo::convert_to_yahoo_ticker(order.ticker.clone());

        // multiplying fill prices by respective fx rate
        stats::fx_adjust(&order.ticker, matcher_date, &mut order.fillPrice, &fx_history);

        // filtering out cancelled or rejected orders
        if order.status == String::from("FILLED") {
            process_order(&order, &mut portfolio_t, &mut ticker_history, &mut real_returns, &mut cash_flows, *time_range.last().unwrap());
        } else {};

        // set portoflio history's element to a correct pair of {Date: portfolio_t}
        let index = time_range.iter().position(|&r| r == matcher_date).expect("time range has no such date");
        portfolio_history[index] = (matcher_date, portfolio_t.clone());
        
    };
    // #########################################################

    



    // PARSING DIVIDENDS #######################################
    let mut blarg: BTreeMap<NaiveDate, f64> = BTreeMap::new();
    let mut total_dividends: f64 = 0.0;
    let mut cum_dividends: HashMap<NaiveDate, f32> = HashMap::new();

    for dividend in dividend_history{
        let date = match NaiveDate::from_str(&dividend.paidOn) {
            Ok(v) => v,
            Err(_) => continue,
        };
        let amount = dividend.amount;
        cash_flows.entry(date).and_modify(|cf| *cf += amount).or_insert(amount);
        dividend_library.entry(dividend.ticker).and_modify(|cf| *cf += amount).or_insert(amount);
        blarg.entry(date).and_modify(|cf| *cf += amount).or_insert(amount);
        total_dividends += amount;
        cum_dividends.entry(date).insert_entry(total_dividends as f32);
    }
    let dividend_history: BTreeMap<NaiveDate, f64> = blarg;
    let mut cum_dividends = hashmap_to_sorted_vec(cum_dividends);
    interpolate(&mut cum_dividends);

    // println!("{:#?}", dividend_library);
    // #########################################################





    // GETTING STOCK PRICES FROM YAHOO #########################
    println!("\n     ticker               lifetime:");
    
    for (ticker, (date1, date2)) in ticker_history.into_iter()  {   // conversion is fine since order does not matter for price lookup
        
        println!("    {:?},from {:?} to {:?}", ticker, date1, date2);
        let mut single_ticker_history = match yahoo::get_prices(&ticker, date1, date2, &mut custom_tickers) {
            Ok(res) => res,
            Err(e) => panic!("Import from yahoo failed with error code: {}", e)
        };

        // multiplying yahoo prices by respective fx rate
        for (date, price) in single_ticker_history.iter_mut() {  // arbitrary order of iteration, but lookup in fx is still via keys so no problem
            stats::fx_adjust(&ticker, *date, price, &fx_history);
        }
        
        complete_prices.insert(ticker, single_ticker_history); 
    };
    // fill in missing weekend prices using Friday prices
    stats::interpolate_weekends(&mut complete_prices);
    //##########################################################





    // UNREALISED RETURNS ######################################
    // portfolio_history is "sparse", so days where it wasn't changed are empty
    // calculate_returns will just infer that empty day portfolio is same as last modified day's one
    let (return_history, cb_mv_history) = match stats::calc_unreal_returns(&portfolio_history, complete_prices, dividend_history) {
        Some((v, b)) => (v,b),
        None => panic!("Calculating returns failed, check dividends arrived")
    };

    // shadowing
    let return_history: Vec<(NaiveDate, f32)> = stats::hashmap_to_sorted_vec(return_history)
    .into_iter()
    .map(|(date, val)| (date, val as f32))  // convert to f32 for plotters module
    .collect();
    // irrelevant atm: benchmark and absolute cb and mv for beta and other stats to add in the future
    // let cb_mv_history: Vec<(NaiveDate, (f64, f64))> = stats::hashmap_to_sorted_vec(cb_mv_history);
    // let (_, _cb_mv_history): (Vec<_>, Vec<(f64, f64)>) = cb_mv_history.into_iter().unzip();
    // let snp_prices: HashMap<NaiveDate, f64> = yahoo::get_prices("^GSPC", start_date, end_date).expect("couldnt get snp");
    // let mut snp_prices = stats::hashmap_to_sorted_vec(snp_prices);
    // stats::interpolate(&mut snp_prices);
    // let _snp_returns = stats::calculate_benchmark_returns(snp_prices);
    //##########################################################





    // REALISED RETURNS #######################################
    let mut real_returns: Vec<(NaiveDate, (f64, f64))> = stats::hashmap_to_sorted_vec(real_returns)
    .into_iter()
    .scan((0.0, 0.0), |state, (date, (a, b))| {  // like a fold, or cumsum over the (market val, cost_basis) tuple
        state.0 += a;
        state.1 += b;
        Some((date, *state))
    })
    .collect();

    let temp = match real_returns.last() {
        Some(v) => v,
        None => &(end_date, (0.0, 0.0))
    };

    real_returns.push((end_date, temp.1));                             // stretch returns to today
    real_returns.insert(0, (start_date, (0.0001, 0.0001)));            // stretch returns to root day
    stats::interpolate(&mut real_returns);                             // stretch to correspond to # of days
    let real_returns_abs: Vec<(NaiveDate, f32)> = real_returns.clone().into_iter().map(|(date, (cb, mv))|(date, ((mv - cb) as f32))).collect();

    // switches tuple (market val, cost basis) into single (real_return)
    let real_returns_rel: Vec<(NaiveDate, f32)> = real_returns.clone().into_iter().map(|(date, (cb, mv))|(date, ((mv/cb - 1.0)*100.0) as f32)).collect();
    let _just_real_returns: Vec<f32> = stats::strip_dates(real_returns_rel.clone());
    // ########################################################





    // MONEY-WEIGHTED RETURNS #################################

    // let file = File::create("test.json").expect("could not create test file");
    // serde_json::to_writer(&file, &cash_flows).unwrap();

    // let file = File::create("test2.json").expect("could not create test file");
    // serde_json::to_writer(&file, &cb_mv_history).unwrap();

    let mut mwrr_returns = Vec::<(NaiveDate, f32)>::new();
    let cb_mv_history = hashmap_to_btree(cb_mv_history);
    let cash_flows = hashmap_to_btree(cash_flows);
    let mut default_mwrr: f64 = 0.0;    // value to fallback to if mwrr algorithm doesn't converge.

    for (date, (_, mv)) in cb_mv_history.iter() {

        // this range + map allows to clone only what is needed for this iteration, i.e. cash_flows[:date]
        // not including cash flows from date itself. otherwise stock sells get double counted as cf and as mv
        let mut cash_flows_plus_mv: Vec<(NaiveDate, f64)> = cash_flows.range(..=date) //+ Days::new(1)
        .map(|(k, v)| (k.clone(), *v))
        .collect();

        // add today's market value as a cash inflow
        if let Some((_, value)) = cash_flows_plus_mv.iter_mut().next_back() {
        *value += mv
        };

        let irr = mwrr(&cash_flows_plus_mv, 0.5).unwrap_or(default_mwrr) * 100.0;
        mwrr_returns.push((*date, irr as f32));
        default_mwrr = irr/100.0;
        // let portfolio_history: HashMap<NaiveDate, HashMap<String, (f64, f64)>> = portfolio_history.clone().into_iter().collect();
        // if irr > 20.0 || (date.year_ce() == (true, 2023) && date.month0() == 9){
        //     println!("date: {:?}, r: {:?}, mv: {:?}", date, irr, mv);
        //     println!("{:?}", portfolio_history.get(date).unwrap())
        // }
    }
    // ########################################################


    

    
    // PRINTING AND PLOTTING TO CONSOLE #######################
    let naivetime_held = end_date - start_date;
    let days_held: f32 = naivetime_held.num_days() as f32;
    let years_held: f32 = (&days_held)/365.0;
    let months_held: i32 = ((&years_held*12.0) as i32) % 12;                                                                              // vvv this is incorrect
    println!("\n \n Found portfolio of {:.} years, {:.} months, and {:.} days.\n", years_held.floor(), months_held, days_held as i32 % 365 - 30*months_held);
    
    // switch to UTF-8 support by default
    if cfg!(target_os = "windows") {
        let _ = Command::new("chcp").arg("65001").status();
    }
    
    println!("\n  Unrealized return, %");
    display_to_console(&return_history, start_date, end_date, 70, 10.0, RGB8::new(254, 255, 110), String::from_str("%").unwrap());
    
    let just_returns: Vec<f32> = stats::strip_dates(return_history);
    let current_return = &just_returns.last().unwrap();
    let annual_return = ((*current_return/100.0 + 1.0).powf(1.0/(&years_held)) - 1.0) * 100.0;
    let daily_returns: Vec<f32> = stats::get_daily_returns(just_returns.clone());
    let (mean, sd, sharpe) = stats::mean_sd_sharpe(&daily_returns);
    let dividend_yield: f64 = total_dividends / ((cb_mv_history.values().map(|(cb, _)| *cb as f64).sum::<f64>()) / (days_held as f64)) / (years_held as f64) * 100.0;
    printallcommands();
    
    loop {
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let command = input.trim().trim();
        
        match command {
            "/s" => {
                clear_last_n_lines(5);
                println!("   _________________________________________");
                println!("  |                       |                 |");
                println!("  | {0: <21} | {1: <15.4} | ", "unrealised PnL(%)", current_return);
                println!("  |                       |                 |");
                println!("  | {0: <21} | {1: <15.4} | ", "APR(%)", annual_return);
                println!("  |                       |                 |");
                println!("  | {0: <21} | {1: <15.4} | ", "std. deviation", sd);
                println!("  |                       |                 |");
                println!("  | {0: <21} | {1: <15.4} | ", "Sharpe ratio", sharpe);
                println!("  |                       |                 |");
                println!("  | {0: <21} | {1: <15.4} | ", "daily avg. return(%)", mean);
                println!("  |                       |                 |");
                println!("   ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾ \n \n");          
            },
            "/r" => {clear_last_n_lines(6);
                println!("\n  Absolute realized return, GBP");
                display_to_console(&real_returns_abs, start_date, end_date, 40, 10.0, RGB8::new(255, 51, 255), String::from_str(" GBP").unwrap());
            },
                
                "/m" =>     {clear_last_n_lines(6);
                println!("\n  Money-Weighted Rate of Return (MWRR), %");
                display_to_console(&mwrr_returns, 
                *cb_mv_history.first_key_value().unwrap().0,
                *cb_mv_history.last_key_value().unwrap().0,
                70, 10.0, RGB8::new(22, 253, 254), String::from_str("%").unwrap());  
            },

            "/d" => {clear_last_n_lines(6);
                println!("\n  Total dividends, GBP");
                display_to_console(&cum_dividends, cum_dividends.first().unwrap().0, end_date, 40, 0.0, RGB8::new(0, 255, 0), String::from(" GBP"));
                println!("\n  ______________________________________");
                println!("  ticker:          total dividends (GBP)");
                println!("  ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾");
                for (ticker, divi) in dividend_library.iter(){
                    println!("  {0: <12}             {1:>.2}", ticker, divi)
                };
                println!("  ______________________________________");
                println!("  Total:                   {:>.2}", total_dividends);
                println!("  ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾");
                println!("  dividend yield on cost (annual): {:.2}%", dividend_yield);
                println!("  ‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾‾");
            }

            "/q" => {println!("  Quitting...");
                break},

            "" => println!("  Enter valid command or /q to quit."),
            _ => {
                clear_last_n_lines(1);
                println!("  Unknown command: {}", command)
                }
        }
        printallcommands()
    }

    // save user-entered cutom tickers back to json file
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true) // ensures overwrite
        .open(path)
        .expect("json writer fail");

    let _ = to_writer(file, &custom_tickers);
}

// ########################################################





// HELPER FUNCS THAT STAY IN MAIN #########################
fn remove_duplicates(orders: &mut Vec<Order>) {
    let mut seen = HashSet::new();
    orders.retain(|order| seen.insert(order.id));
}



fn get_time_range(data: &Vec<Order>) -> Result<Vec<NaiveDate>, Box<dyn Error>> {
    
    let root_date = data.first().ok_or("couldn't get first order")?.dateModified.as_str();    

    // ^^^ last() returns an option, ok_or converts it to result, "?" propagates the error

    let mut start_date = NaiveDate::parse_from_str(&root_date, "%Y-%m-%d")?;
    
    
    let end_date = Utc::now().date_naive();
    
    let mut time_range = Vec::new();
    
    while start_date <= end_date {
        time_range.push(start_date);
        start_date += Duration::days(1);
    }
    
    Ok(time_range)    // return
}



fn process_order(
    order: &Order,
    portfolio_t: &mut HashMap<String, (f64, f64)>,
    ticker_history: &mut HashMap<String, (NaiveDate, NaiveDate)>,
    real_returns: &mut HashMap<NaiveDate, (f64, f64)>,
    cash_flows: &mut HashMap<NaiveDate, f64>,
    last_date: NaiveDate) {

    let q_1 = order.filledQuantity;
    let p_1 = order.fillPrice;
    let date = NaiveDate::from_str(order.dateModified.as_str()).unwrap();
    let ticker = order.ticker.clone();
    
    // log the order as a cash flow
    cash_flows.entry(date).and_modify(|days_cash_flow| *days_cash_flow += (-q_1*p_1)).or_insert(-q_1*p_1);
    
    // log the order's presence in portolios and ticker histories
    match portfolio_t.entry(order.ticker.clone()) {
        Entry::Occupied(mut occupied) => {
            
            let (q_0, p_0) = occupied.get_mut();
            
            if *q_0 + q_1 == 0.0 {                                              // if sold everything
                
                let (keeps_date, _) = ticker_history.get(&ticker).unwrap();
                ticker_history.insert(ticker, (*keeps_date, date));
                
                real_returns.entry(date)
                .and_modify(|cbmv| *cbmv = (cbmv.0 + *p_0*(-q_1), cbmv.1 + p_1*(-q_1)))
                .or_insert((*p_0*(-q_1), p_1*(-q_1)));
            
            occupied.remove();    // removes ticker from portfolio
            
        } else {
            if q_1 >= 0.0 {                                                // if bought some *more*
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
                    
                    
                        real_returns.entry(date)
                        .and_modify(|cbmv| *cbmv = (cbmv.0 + *p_0*(-q_1), cbmv.1 + p_1*(-q_1)))
                        .or_insert((*p_0*(-q_1), p_1*(-q_1)));
                    };
        };
    },
    Entry::Vacant(vacant) => {                                            // if bought some

            
        vacant.insert((q_1, p_1));
        
        ticker_history.entry(ticker.clone())
        .and_modify(|e| e.1 = last_date.clone())
        .or_insert((date.clone(), last_date.clone()));
},
};
}      // returns nothing, just amends portfolio_t and ticker_history in-place








#[test]
fn do_stuff(){
    let file = File::open("test.json").unwrap();
    let reader = BufReader::new(file);
    let cash_flows: HashMap<NaiveDate, f64> = serde_json::from_reader(reader).unwrap();

    let file = File::open("test2.json").unwrap();
    let reader = BufReader::new(file);
    let cb_mv_history: HashMap<NaiveDate, (f64, f64)> = serde_json::from_reader(reader).unwrap();
    let mut mwrr_returns = Vec::<(NaiveDate, f32)>::new();

    let cb_mv_history = hashmap_to_btree(cb_mv_history);
    let cash_flows = hashmap_to_btree(cash_flows);

    let mut default_mwrr: f64 = 0.0;    // value to fallback to if mwrr algorithm doesn't converge.


    for (date, (_, mv)) in cb_mv_history.iter() {

        // this range + map allows to clone only what is needed for this iteration, i.e. cash_flows[:date]
        let mut cash_flows_plus_mv: Vec<(NaiveDate, f64)> = cash_flows.range(..=date) //+ Days::new(2)
        .map(|(k, v)| (k.clone(), *v))
        .collect();

        // add today's market value as a cash inflow
        if let Some((_, value)) = cash_flows_plus_mv.iter_mut().next_back() {
        *value += mv
        };

        let irr = mwrr(&cash_flows_plus_mv, 0.5).unwrap_or(default_mwrr) * 100.0;
        mwrr_returns.push((*date, irr as f32));
        default_mwrr = irr/100.0;
    }

    display_to_console(&mwrr_returns, *cb_mv_history.first_key_value().unwrap().0,
        *cb_mv_history.last_key_value().unwrap().0,
        70, 10.0, RGB8::new(254, 245, 116), String::from_str("%").unwrap());
    // // let cash_flows = serde_json::to_vec_pretty(&file);
    // let file = File::create("test.json").expect("could not create test file");
    // serde_json::to_writer(&file, &cash_flows);
    // println!("XIRRRRRRRRRRRRRRRRRRRRRRR with no divis = {:.6}%", mwrr(&cash_flows, 0.5).expect("the xirr failed, not the interpolate") * 100.0);
    }
