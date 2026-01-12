use adif_parser::{AdifError, parse_adi};
use std::env;
use std::fs;
use std::process;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <adif_file>", args[0]);
        eprintln!("  Parse and display contents of an ADIF file");
        process::exit(1);
    }

    let filename = &args[1];

    if let Err(e) = run(filename) {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run(filename: &str) -> Result<(), AdifError> {
    let content = fs::read_to_string(filename)?;
    let adif_file = parse_adi(&content)?;

    // Display header information
    println!("=== ADIF File: {} ===", filename);
    println!();

    if let Some(version) = &adif_file.header.adif_version {
        println!("ADIF Version: {}", version);
    }
    if let Some(program) = &adif_file.header.program_id {
        print!("Created by: {}", program);
        if let Some(ver) = &adif_file.header.program_version {
            print!(" v{}", ver);
        }
        println!();
    }
    if let Some(timestamp) = &adif_file.header.created_timestamp {
        println!("Created: {}", format_timestamp(timestamp));
    }

    if !adif_file.header.preamble.trim().is_empty() {
        println!();
        println!("Header comments:");
        for line in adif_file.header.preamble.lines().take(5) {
            let trimmed = line.trim();
            if !trimmed.is_empty() {
                println!("  {}", trimmed);
            }
        }
    }

    println!();
    println!("=== Records: {} QSOs ===", adif_file.len());
    println!();

    // Display records in a table format
    if !adif_file.is_empty() {
        println!(
            "{:<12} {:<10} {:<6} {:<10} {:<8} {:<6} {:<6}",
            "Call", "Date", "Time", "Freq", "Band", "Mode", "RST"
        );
        println!("{}", "-".repeat(70));

        for record in adif_file.iter() {
            let call = record.call().unwrap_or("-");
            let date = record
                .qso_date()
                .map(format_date)
                .unwrap_or_else(|| "-".to_string());
            let time = record
                .time_on()
                .map(format_time)
                .unwrap_or_else(|| "-".to_string());
            let freq = record.freq().unwrap_or("-");
            let band = record.band().unwrap_or("-");
            let mode = record.mode().unwrap_or("-");
            let rst = record.rst_sent().unwrap_or("-");

            println!(
                "{:<12} {:<10} {:<6} {:<10} {:<8} {:<6} {:<6}",
                truncate(call, 12),
                date,
                time,
                truncate(freq, 10),
                truncate(band, 8),
                truncate(mode, 6),
                truncate(rst, 6)
            );
        }
    }

    println!();
    println!("Total: {} QSO(s)", adif_file.len());

    Ok(())
}

fn format_date(date: &str) -> String {
    if date.len() == 8 {
        format!("{}-{}-{}", &date[0..4], &date[4..6], &date[6..8])
    } else {
        date.to_string()
    }
}

fn format_time(time: &str) -> String {
    if time.len() >= 4 {
        format!("{}:{}", &time[0..2], &time[2..4])
    } else {
        time.to_string()
    }
}

fn format_timestamp(ts: &str) -> String {
    if ts.len() >= 8 {
        let date_part = format_date(&ts[0..8]);
        if ts.len() >= 12 {
            let time_part = format_time(&ts[8..12]);
            format!("{} {}", date_part, time_part)
        } else {
            date_part
        }
    } else {
        ts.to_string()
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
