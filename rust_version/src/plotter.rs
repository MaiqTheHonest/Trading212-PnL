use rgb::RGB8;
use textplots::{Chart, ColorPlot, LabelBuilder, LabelFormat, Shape, TickDisplay, TickDisplayBuilder};
use chrono::{Duration, NaiveDate};
use std::{collections::HashMap, io::{self, Write}};
use piechart::{Chart as PieChart, Color, Data, Style};
use crate::stats::hashmap_to_sorted_vec;

pub fn display_to_console(
    data_to_plot_1: &Vec<(NaiveDate, f32)>,
    start_date: NaiveDate,
    end_date: NaiveDate,
    size: u32,
    y_offset: f32,
    colour: RGB8,
    units: String) {



    let mut points: Vec<(f32, f32)> = data_to_plot_1
        .iter()
        .map(|(date, value)| {
            let day_number = (*date - start_date).num_days() as f32; 
            (day_number, (value + 0.0* (day_number as f32) / (data_to_plot_1.len() as f32)) as f32)
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
    


    Chart::new_with_y_range(3*size, 2*size, points.last().unwrap().0/-25.0, points.last().unwrap().0/1.0, myround(ymin, 5.0)-y_offset, myround(ymax, 5.0)+10.0)
        .linecolorplot(&Shape::Lines(&points), colour)
        .x_label_format(LabelFormat::Custom(Box::new(move |val| {
            if val <= 1.0 { format!("  {}{}{}", start_date.to_string(), (0..(size*2/3 - 10)).map(|_| " ").collect::<String>(), mid_date.to_string()) } 
            else if val >= 2.0 {format!("{}", end_date.to_string()) } 
            else { "".to_string() }
        })))

        .y_label_format(LabelFormat::Custom(Box::new(move |value| {format!("{:.1} {}", value, units)})))
        .y_tick_display(TickDisplay::Dense)
        .nice();

}




pub fn printallcommands() {
    println!("\n  /s      view portfolio statistics            /m      view MWRR (Trading 212 returns)");
    println!("  /r      view realized returns                /d      view dividend statistics");
    println!("  /f      view fees and taxes                  /q      quit\n");
}



pub fn clear_last_n_lines(n: u8) {
    let mut stdout = io::stdout();
    for _ in 0..n {
        // move cursor up a line
        write!(stdout, "\x1B[1A").unwrap();
        // clear the line
        write!(stdout, "\x1B[2K").unwrap();
    }
    stdout.flush().unwrap();
}


pub fn draw_pie(hm: HashMap<String, f32>){
    let mut data = vec![];
    let mut palette = [Color::RGB(190, 190, 0), Color::RGB(220, 5, 5), Color::Green, Color::Black].iter().cycle();
    let mut ordered_hm: Vec<(String, f32)> = hm.into_iter().collect();
    ordered_hm.sort_by(|a, b| a.1.total_cmp(&b.1));

    for (key, val) in ordered_hm.iter(){
        let twodpval = (val * 100.0).round() / 100.0 * -1.0;
        data.push(Data { label: key.into(), value: twodpval, color: Some(Style::from(*palette.next().unwrap())), fill: '•' });
        
    } 

    PieChart::new()
    .radius(5)
    .aspect_ratio(4)
    .legend(true)
    .draw(&data);
}