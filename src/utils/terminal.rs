use crate::PAIRS;

use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, TimeZone, Utc};
use dialoguer::Select;
use indicatif::{ProgressBar, ProgressStyle};
use inquire::Text;
use std::{
        fs::{canonicalize, create_dir_all},
        path::PathBuf,
        process::exit,
};
use tokio::sync::mpsc::Receiver;

pub fn clear_terminal() { print!("{esc}c", esc = 27 as char); }

// Function to display a progress bar
pub async fn show_progress(mut rx: Receiver<usize>)  {
    let pb = ProgressBar::new(100);

    // Style
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}% ({eta})").unwrap()
    );

    // Spawn the progress bar
    pb.set_position(0);

    // Wait for an update from the channel
    // And update the progress bar
    while let Some(value) = rx.recv().await {
        if value == 0 {
            pb.finish_with_message("Data downloaded successfully");

            break;
        }
        pb.inc(value as u64);
    }
}

pub async fn choose_pair() -> String {
    clear_terminal();

    // Get the list of pairs from the PAIRS static variable
    // Ordered by the name of the pair
    let pairs = {
        let pairs = PAIRS.lock().await;
        let mut keys = pairs.keys().cloned().collect::<Vec<String>>();
        keys.sort();

        keys
    };

    if pairs.is_empty() {
        println!("No pairs available");
        exit(1);
    }

    // Create a new select prompt
    let selection = Select::new()
        .with_prompt("Select a currency pair")
        .default(0)
        .items(&pairs)
        .interact()
        .unwrap();

    pairs[selection].to_string()
}

pub async fn choose_dates(pair: String) -> (DateTime<Utc>, DateTime<Utc>) {
    clear_terminal();

    // Get the minimum date from the PAIRS static variable
    let min_date = {
        let pairs = PAIRS.lock().await;
        let pair = pairs.get(&pair).unwrap();

        pair.naive_utc()
    };
    let max_date = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap().and_time(NaiveTime::from_hms_opt(23, 59, 59).unwrap());

    // Initialize the beginning and end dates
    let beginning_date;
    let end_date;

    // While the user doesn't enter a valid date
    // Ask for the end date
    loop {
        let date_input = Text::new(&format!("Enter beginning date (YYYY-MM-DD), between {} and {}: ", &min_date.format("%Y-%m-%d").to_string(), &max_date.format("%Y-%m-%d").to_string()))
            .prompt().unwrap();

        match NaiveDateTime::parse_from_str(&format!("{} 00:00:00", date_input), "%Y-%m-%d %H:%M:%S") {
            Ok(date) if date >= min_date && date <= max_date => {
                beginning_date = date;
                break;
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }

    // While the user doesn't enter a valid date
    // Ask for the end date
    loop {
        let date_input = Text::new(&format!("Enter end date (YYYY-MM-DD), between {} and {}: ", &beginning_date.format("%Y-%m-%d").to_string(), &max_date.format("%Y-%m-%d").to_string()))
            .prompt().unwrap();

        match NaiveDateTime::parse_from_str(&format!("{} 23:59:59", date_input), "%Y-%m-%d %H:%M:%S") {
            Ok(date) if date >= beginning_date && date <= max_date => {
                end_date = date;
                break;
            }
            Ok(_) => {}
            Err(_) => {}
        }
    }

    // Convert the dates to UTC
    return (
        Utc.from_utc_datetime(&beginning_date),
        Utc.from_utc_datetime(&end_date),
    );
}

pub fn choose_destination() -> PathBuf {
    clear_terminal();

    // Ask the user for the destination directory
    let destination = Text::new("Where do you want to save the data?")
        .prompt()
        .unwrap();

    // Convert the string to a PathBuf
    let path = PathBuf::from(destination);

    let abs_path = match canonicalize(&path) {
        Ok(p) => p,
        Err(_) => {
            // Si la canonicalisation échoue (ex: dossier pas encore créé),
            // on peut essayer de créer le dossier et re-canonicaliser
            if let Err(e) = create_dir_all(&path) {
                eprintln!("Failed to create directory {}: {}", path.display(), e);
                exit(1);
            }
            canonicalize(&path).unwrap_or_else(|e| {
                eprintln!("Failed to canonicalize directory after creation: {}", e);
                exit(1);
            })
        }
    };

    return abs_path;
}

pub fn choose_datatype() -> String {
    clear_terminal();

    let data_types = vec!["csv", "parquet" ];

    // Create a new select prompt
    let selection = Select::new()
        .with_prompt("Select a data type")
        .default(0)
        .items(&data_types)
        .interact()
        .unwrap();

    data_types[selection].to_string()
}