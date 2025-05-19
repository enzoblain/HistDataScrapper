use histdatascraper::data::handler::download_data;
use histdatascraper::data::pairs::build_pairs;
use histdatascraper::utils::terminal::{choose_datatype, choose_dates, choose_destination, choose_pair, show_progress};

use tokio::{task::spawn, sync::mpsc::channel};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    build_pairs().await;

    // Create a channel to send the progress
    let (tx, rx) = channel(100);
    
    // Let the user choose a pair
    // the dates
    // and the destination
    let pair = choose_pair().await;
    let (from_date, to_date) = choose_dates(pair.clone()).await;
    let data_dir = choose_destination();
    let data_type = choose_datatype();

    // Create the main task with the sender
    // and the receiver
    let _ = spawn(download_data(pair.clone(), from_date, to_date, data_dir, data_type, tx));

    // Create a task to show the progress
    let rx_task = spawn(show_progress(rx));
    rx_task.await.unwrap();

    Ok(())
}