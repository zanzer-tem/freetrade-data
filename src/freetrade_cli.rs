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
        //Mode::INTERACTIVE => prompt_for_symbols(&freetrade_data) , 
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
        let data_row = line.unwrap();
        let symbol_position = data_row.find(",").unwrap();
        let symbol = &data_row[0..symbol_position];
        let available = freetrade_data.is_isa_eligible(&symbol);
        if available {
            println!("{}", style(symbol).magenta());
            writeln!(buf_writer, "{},", symbol).expect("unable to write");
        }
    }
    Ok(())
}

fn interactive_session(freetrade_data: &FreetradeData) {
    loop {
        println!("{}", "Tool:\n1: Symbol Lookup\n2: Exchange Listings\n");
        let choice: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Option")
                .interact_text()
                .unwrap();

        match choice.as_str() {
            "1" => prompt_for_symbols(freetrade_data),
            "2" => prompt_for_exchange(freetrade_data),
            "q" => break,
            _=> interactive_session(freetrade_data)
        }
    }
}

fn prompt_for_exchange(freetrade_data: &FreetradeData) {
    let choice: String = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Exchange (NYSE, NASDAQ, LSE)")
                .interact_text()
                .unwrap();
    let exchange = match choice.to_uppercase().as_str() {
        "NASDAQ" => Exchange::NASDAQ,
        "NYSE" => Exchange::NYSE,
        "LSE" => Exchange::LSE,
        _=> Exchange::UNKNOWN
    };

    if matches!(exchange, Exchange::UNKNOWN) {
        println!("{}", style("Unknown Exchange").magenta());
        prompt_for_exchange(freetrade_data);
    } else {
        let exchange_symbols: Vec<&SymbolData> = freetrade_data.symbols_in_exchange(exchange);
        println!("Symbols in {} ({})", style(&exchange).magenta(), style(&exchange_symbols.len()).magenta());
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

        let available = freetrade_data.is_isa_eligible(&symbol);

        if available {
            let symbol:&SymbolData = freetrade_data.symbol(&symbol).unwrap();
            println!("{} {} {}\n{} {}", style("Ticker: ").cyan(),style(&symbol.symbol.symbol).green(), style(&symbol.symbol.name).magenta(), style("Exchange: ").cyan(),style(&symbol.exchange).magenta());
        } else {
            println!("{}", style("Symbol unavailable").cyan());
        }
    }
}
