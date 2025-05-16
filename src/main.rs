use histdatascraper::data::handler::{build_pairs, downloader};
use histdatascraper::utils::browser::{close_browser, launch_browser};
use histdatascraper::utils::terminal::{choose_datatype, choose_dates, choose_destination, choose_pair, show_progress};
use histdatascraper::utils::utils::find_available_port;

use chrono::Datelike;
use tokio::task;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Search for an available port in the range 9500-9999
    if !find_available_port(9500, 9999).await {
        println!("Failed to find an available port");

        return Ok(());
    }

    build_pairs().await;

    // Launch the browser
    launch_browser().await.unwrap();

    // Create a channel to send the progress
    let (tx, rx) = tokio::sync::mpsc::channel(1000);
    
    // Let the user choose a pair
    // and the dates
    let pair = choose_pair().await;
    let (from_date, to_date) = choose_dates(pair.clone()).await;

    let year_duration = to_date.year() - from_date.year();

    let data_dir = choose_destination();
    let data_type = choose_datatype();

    // Create the main task with the sender
    // and the receiver
    let _ = task::spawn(downloader(tx, pair.to_string(), from_date, to_date, data_dir.clone(), data_type.clone()));
    let rx_task = task::spawn(show_progress(rx, year_duration as u64));

    rx_task.await.unwrap();

    // Wait for the downloader task to finish
    // and then close the browser
    close_browser().await.unwrap();

    println!("Download successful, data saved in: {}/{}.{}", data_dir.display(), pair, data_type);

    Ok(())
}