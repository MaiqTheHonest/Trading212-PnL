use rgb::RGB8;
use textplots::{AxisBuilder, Chart, ColorPlot, LabelBuilder, Plot, Shape, TickDisplayBuilder};
use chrono::{Duration, NaiveDate, Utc};
// use serde::de::Error;
use std::{collections::{hash_map::Entry, HashMap}, error::Error, str::FromStr};


pub fn display_to_console(data_to_plot: &HashMap<NaiveDate, f64>, start_date: NaiveDate) {


    // let mut data_to_plot: HashMap<NaiveDate, f64> = HashMap::new();

    let mut points: Vec<(f32, f32)> = data_to_plot
        .iter()
        .map(|(&date, &value)| {
            let day_number = (date - start_date).num_days() as f32; 
            (day_number, value as f32)
        })
        .collect();
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let mut y_returns: Vec<f32> = points.iter().map(|(_, y)| *y).collect();    // mind the deref

    let (ymin, ymax) = y_returns.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY), 
        |(min, max), &val| (min.min(val), max.max(val))
    );

    Chart::new_with_y_range(300, 200, (points.last().unwrap().0/-25.0), points.last().unwrap().0/1.0, ymin-10.0, ymax+10.0)
        .linecolorplot(&Shape::Lines(&points), RGB8::new(254, 245, 116))
        .y_label_format(textplots::LabelFormat::Value)
        .display();
    // .lineplot(&Shape::Continuous(Box::new(|x| -3.0)))
    
    // dots.display();
}