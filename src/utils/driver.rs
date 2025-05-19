use reqwest;
use std::{
        process::{Command, Stdio}, 
        time::Duration
};
use tokio::time::sleep;

pub async fn launch_driver(port: usize) -> Result<(), String> {
    // Launch the browser
    let _ = Command::new("chromedriver")
        .arg(format!("--port={}", port))
        .stdout(Stdio::null())
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

pub async fn close_driver(port: usize) -> Result<(), String> {
    // Get the process ID of the browser
    let output = Command::new("lsof")
        .args(&["-ti", &format!(":{}", port)]) 
        .output().unwrap();

    // If we find out
    // kill the process
    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);

        for pid in stdout.lines().rev() {
            
            let output = Command::new("kill")
            .arg("-9")
            .arg(pid)
            .output();

            if output.is_err() {
                return Err(format!("Failed to kill process: {}", pid));
            }
        }

    }

    Ok(())
}