use crate::PORT;

use reqwest;
use std::{process::Command, time::Duration};
use tokio::time::sleep;

pub async fn launch_browser() -> Result<(), String> {
    // Lock the PORT mutex to get the available port
    let port = {
        let port_lock = PORT.lock().await;
        *port_lock
    };

    // Launch the browser
    let _ = Command::new("chromedriver")
        .arg(format!("--port={}", port))
        .spawn()
        .map_err(|e| format!("Failed to launch browser: {}", e))?;

    // Wait for the browser to be ready
    let client = reqwest::Client::new();
    loop {
        // Check if the browser is ready
        // by sending a request to the status endpoint
        match client.get(format!("http://localhost:{}/status", port)).send().await {
            Ok(resp) if resp.status().is_success() => {
                return Ok(());
            }
            _ => {
                // If the request fails, wait for a bit and try again
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

pub async fn close_browser() -> Result<(), String> {
    // Close the browser
    let child_result = Command::new("pkill")
        .arg("chromedriver")
        .spawn();

    // Check for errors
    match child_result {
        Ok(_) => {
            Ok(())
        }
        Err(e) => {
            Err(format!("Failed to close browser: {}", e))
        }
    }
}