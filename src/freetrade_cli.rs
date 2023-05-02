use console::style;
use dialoguer::{theme::ColorfulTheme, Input};
use freetrade_data::freetrade_data::FreetradeData;
use std::env;
use std::fs::File;
use std::io::{ BufWriter, BufReader, BufRead};
use std::io::Write;
use chrono::prelude::*;
use freetrade_data::freetrade_data::*;


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
    let freetrade_data: FreetradeData = FreetradeData::new();
    let total_symbols: usize = freetrade_data.symbols().len();
    println!("{} {}", style("Total symbols found: ").cyan(), total_symbols);
    println!("{}", style("'q' to quit").cyan());
    
    
    
    match mode {

        Mode::INTERACTIVE => interactive_session(&freetrade_data) , 
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
        let data_row = line?;
        let symbol_position = data_row.find(",").unwrap();
        let symbol = &data_row[0..symbol_position];
        let matches = freetrade_data.isa_eligible_symbol(symbol);
        for s in matches {
            println!("{}", style(&s.symbol).magenta());
            writeln!(buf_writer, "{},", s.symbol).expect("unable to write");
        }
    }
    Ok(())
}

fn interactive_session(freetrade_data: &FreetradeData) {
    loop {
        println!("{}", "Tool:\n1: Symbol Lookup\n");
        let choice: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Option")
                .interact_text()
                .unwrap();

        match choice.as_str() {
            "1" => prompt_for_symbols(freetrade_data),
            "q" => break,
            _=> interactive_session(freetrade_data)
        }
    }
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

        let matches = freetrade_data.isa_eligible_symbol(&symbol);

        match matches.len() {
            0 => println!("{}", style("Symbol unavailable").cyan()),
            _ => for m in matches  {
                    println!("{} {} {}\n{} {}", style("Ticker: ").cyan(),style(&m.symbol).green(), style(&m.title).magenta(), style("Exchange: ").cyan(),style(&m.mic).magenta());
                }
        };
    }
}
