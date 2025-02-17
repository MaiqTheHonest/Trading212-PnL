use rgb::RGB8;
use textplots::{Chart, ColorPlot, LabelBuilder, LabelFormat, Shape, TickDisplay, TickDisplayBuilder};
use chrono::{Duration, NaiveDate};
use std::collections::HashMap;


pub fn display_to_console(data_to_plot: &HashMap<NaiveDate, f64>, start_date: NaiveDate, end_date: NaiveDate) {


    // let mut data_to_plot: HashMap<NaiveDate, f64> = HashMap::new();

    let mut points: Vec<(f32, f32)> = data_to_plot
        .iter()
        .map(|(&date, &value)| {
            let day_number = (date - start_date).num_days() as f32; 
            (day_number, value as f32)
        })
        .collect();
    points.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
    

    let y_returns: Vec<f32> = points.iter().map(|(_, y)| *y).collect();    // mind the deref

    let (ymin, ymax) = y_returns.iter().fold(
        (f32::INFINITY, f32::NEG_INFINITY), 
        |(min, max), &val| (min.min(val), max.max(val))
    );



    let mid_date = start_date + Duration::days((end_date-start_date).num_days()/2);


    fn myround(x:f32, base: f32) -> f32{
        (x/base).round() * base
    }
    


    Chart::new_with_y_range(300, 200, points.last().unwrap().0/-25.0, points.last().unwrap().0/1.0, myround(ymin, 5.0)-10.0, myround(ymax, 5.0)+10.0)
        .linecolorplot(&Shape::Lines(&points), RGB8::new(254, 245, 116))
        .x_label_format(LabelFormat::Custom(Box::new(move |val| {
            if val <= 1.0 { format!("  {}                                                            {}", start_date.to_string(), mid_date.to_string()) } 
            else if val >= 2.0 { end_date.to_string() } 
            else { "".to_string() }
        })))

        .y_label_format(LabelFormat::Custom(Box::new(|value| format!("{:.1}%", value))))
        .y_tick_display(TickDisplay::Dense)
        .nice();

}

