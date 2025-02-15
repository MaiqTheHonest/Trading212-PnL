use rgb::RGB8;
use textplots::{Chart, ColorPlot, Plot, Shape};
use chrono::{Duration, NaiveDate, Utc};
// use serde::de::Error;
use std::{collections::{hash_map::Entry, HashMap}, error::Error, str::FromStr};

pub fn display_to_console(data_to_plot: HashMap<NaiveDate, f64>, start_date: NaiveDate) {


    // let mut data_to_plot: HashMap<NaiveDate, f64> = HashMap::new();

    let mut sorted_points: Vec<(f32, f32)> = data_to_plot
        .iter()
        .map(|(&date, &value)| {
            let day_number = (date - start_date).num_days() as f32; 
            (day_number, value as f32)
        })
        .collect();

    Chart::new_with_y_range(200, 100, -5.0, sorted_points.last().unwrap().0, -100.0, 100.0)
    .linecolorplot(&Shape::Points(&sorted_points), RGB8::new(255, 0, 0))
    .display();
    // .lineplot(&Shape::Continuous(Box::new(|x| -3.0)))
    
    // dots.display();
}