use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use freetrade_data::freetrade_data::FreetradeData;
use std::env;
use std::fs::File;
use std::io::{ BufWriter, BufReader, BufRead};
use std::io::Write;
use chrono::prelude::*;
#[derive(Debug)]
enum Mode {
    INTERACTIVE,
    AUTO
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let mode:Mode = match args.len() {
        2 => Mode::AUTO,
        _ => Mode::INTERACTIVE
    };
    
    println!("{}", style("Freetrade CLI").magenta());
    println!("{} {:?}", style("Mode: ").cyan(), mode);
    println!("{}", style("'q' to quit").cyan());
    
    let freetrade_data = FreetradeData::new();
    match mode {
        Mode::INTERACTIVE => prompt_for_symbols(&freetrade_data) , 
        Mode::AUTO => read_symbols_from_file(&freetrade_data, &args[1]).unwrap()
    }
}

fn read_symbols_from_file(freetrade_data: &FreetradeData, filename: &str) -> Result<(), Box<dyn std::error::Error>>{
    println!("Reading from file");
    let file = File::open(filename).expect("file not found!");
    let output_file_name = Utc::now().to_string();
    let output_file = File::create(output_file_name).expect("Cannot create output file");
    let mut buf_writer = BufWriter::new(output_file);
    let buf_reader = BufReader::new(file);
    for line in buf_reader.lines().skip(1) {
        let data_row = line.unwrap();
        let symbol_position = data_row.find(",").unwrap();
        let symbol = &data_row[0..symbol_position];
        let available = freetrade_data.contains(&symbol, true);
        if available {
            println!("{}", style(symbol).magenta());
            writeln!(buf_writer, "{},", symbol).expect("unable to write");
        }
    }
    Ok(())
}

fn prompt_for_symbols(freetrade_data: &FreetradeData){
    loop {
        let symbol: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Symbol")
            .interact_text()
            .unwrap();

        if symbol == "q" {
            break;
        }

        let available = freetrade_data.contains(&symbol, true);

        println!("{}", style(available).cyan());
    }
}
