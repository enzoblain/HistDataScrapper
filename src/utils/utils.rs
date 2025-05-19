use crate::NUMBER_OF_SIMULTANEOUS_TASKS;

use directories::UserDirs;
use once_cell::sync::Lazy;
use std::{
        fs::{create_dir_all, metadata, remove_file, File},
        io::{self, copy},
        net::TcpListener,
        path::Path,
        process::exit,
        thread::sleep,
        time::Duration,
};
use tokio::sync::Mutex;
use zip::ZipArchive;

// Store the port in a static variable
// This is a global variable that will be used to store the port
pub static PROGRESS: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(0.0));

// Because running multiple tasks at the same time
// may want to use the same port
// We need to store the ports in use
pub static PORT_IN_USE: Lazy<Mutex<Vec<usize>>> = Lazy::new(|| Mutex::new(vec![]));

pub async fn find_available_port(from: usize, to: usize) -> usize {
    // Check every port in the range from `from` to `to`
    for port in from..to {
        // Try to bind to the port
        // If it succeeds, then the port is available
        if let Ok(listener) = TcpListener::bind(format!("127.0.0.1:{}", port)) {
            let _ = listener.set_nonblocking(true);
            drop(listener);

            // Check if the port is already in use
            // If it is, then continue to the next port
            // If it isn't, then return the port
            let mut ports_in_use = PORT_IN_USE.lock().await;
            if ports_in_use.contains(&port) {
                continue;
            }
            
            ports_in_use.push(port);

            return port;
        }
    }

    println!("No available port found in the range {}-{}", from, to);
    exit(1)
}

// Return the download directory of the current user
// We can't hardcode the path to the download directory because it's different on each system
pub fn get_download_dir() -> Result<String, String> {
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(download_dir) = user_dirs.download_dir() {
            return Ok(download_dir.to_string_lossy().to_string());
        }
    }

    Err("Failed to get download directory".to_string())
}

// Unzip the file to the wanted directory
pub fn unzip_file(zip_path: &str) -> io::Result<()> {
    // Open the zip file
    // Create a new zip archive
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Iterate through the files in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(zip_path.strip_suffix(".zip").unwrap()).join(file.name());

        if (*file.name()).ends_with('/') {
            // If it's a directory
            // Create the directory if it doesn't exist
            create_dir_all(&outpath)?;
        } else {
            // if it's a file
            // Check if the parent directory exists
            // If it doesn't exist, create it
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    create_dir_all(&p)?;
                }
            }

            // Create the file 
            // Copy the content of the file to the new file
            let mut outfile = File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
        }
    }

    // Remove the zip file
    remove_file(zip_path)?;

    Ok(())
}

// Function to wait until the file is downloaded
pub fn wait_until_file_downloaded(path: &str) {
    let mut last_size = 0;

    loop {
        if Path::new(path).exists() {
            // Get the metadata of the file
            // Get the size of the file
            let metadata = metadata(path).unwrap();
            let size = metadata.len();

            // Check if the file has stopped growing
            // If it is then we can assume that the download is complete 
            // and we can break the loop
            if size == last_size && size > 0 {
                break;
            }

            last_size = size;
        }

        // Sleep for a short duration to avoid busy waiting
        sleep(Duration::from_micros(100));
    }
}

// This function serves use to split the date range into the number of tasks
// So we can download the data in parallel
pub fn calculate_split(from_year: usize, year_duration: usize) -> Vec<Vec<usize>> {

    // We calculate the minmum number of years for each task
    // And the surplus years
    let base = year_duration / NUMBER_OF_SIMULTANEOUS_TASKS;
    let rest = year_duration % NUMBER_OF_SIMULTANEOUS_TASKS;

    // We calculate the number of years for each task
    let split: Vec<usize> = (0..NUMBER_OF_SIMULTANEOUS_TASKS)
        .map(|i| if i < rest { base + 1 } else { base })
        .collect();

    // Then we create a vector of years for each task
    // So we get the years for each task instead of the number of years
    let mut current_year = from_year;
    let final_split: Vec<Vec<usize>> = split
        .iter()
        .map(|&rep| {
            let years: Vec<usize> = (current_year..current_year + rep).collect();
            current_year += rep;
            years
        })
        .collect();

    final_split
}

// We want to get the pourcentage of the progress
// So we need to calculate the pourcentage of the progress
pub fn calculate_progress_weight(year_duration: usize) -> usize {
    // 2 times because we download the data and then parse it
    // And +1 because we need to save the data
    let progress_weight = year_duration * 2 + 1;

    (100 / progress_weight) as usize
}