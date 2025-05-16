use directories::UserDirs;
use once_cell::sync::Lazy;
use std::{fs::{self, File}, io, path::Path, thread, time::Duration};
use tokio::sync::Mutex;
use zip::ZipArchive;

// Store the port in a static variable
// This is a global variable that will be used to store the port
pub static PORT: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));
pub static PROGRESS: Lazy<Mutex<f64>> = Lazy::new(|| Mutex::new(0.0));

pub async fn find_available_port(from: usize, to: usize) -> bool {
    // Check every port in the range from `from` to `to`
    for port in from..to {
        // Try to bind to the port
        // If it succeeds, then the port is available
        if let Ok(listener) = std::net::TcpListener::bind(format!("127.0.0.1:{}", port)) {
            let _ = listener.set_nonblocking(true);
            
            // Then update the global port variable
            let mut port_lock = PORT.lock().await;
            *port_lock = port;

            return true;
        }
    }
    
    // If no port is available, return false
    return false;
}

// Return the download directory of the current user
// We can't hardcode the path to the download directory because it's different on each system
pub fn get_download_dir() -> Option<String> {
    if let Some(user_dirs) = UserDirs::new() {
        if let Some(download_dir) = user_dirs.download_dir() {
            return Some(download_dir.to_string_lossy().to_string());
        }
    }

    None
}

// Unzip the file to the wanted directory
pub fn unzip_file(zip_path: &str, dest_dir: &str) -> io::Result<()> {
    // Open the zip file
    // Create a new zip archive
    let file = File::open(zip_path)?;
    let mut archive = ZipArchive::new(file)?;

    // Iterate through the files in the archive
    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = Path::new(dest_dir).join(file.name());

        if (*file.name()).ends_with('/') {
            // If it's a directory
            // Create the directory if it doesn't exist
            fs::create_dir_all(&outpath)?;
        } else {
            // if it's a file
            // Check if the parent directory exists
            // If it doesn't exist, create it
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }

            // Create the file 
            // Copy the content of the file to the new file
            let mut outfile = File::create(&outpath)?;
            io::copy(&mut file, &mut outfile)?;
        }
    }

    // Remove the zip file
    fs::remove_file(zip_path)?;

    Ok(())
}

// Function to wait until the file is downloaded
pub fn wait_until_file_downloaded(path: &str) {
    let mut last_size = 0;

    loop {
        if Path::new(path).exists() {
            // Get the metadata of the file
            // Get the size of the file
            let metadata = fs::metadata(path).unwrap();
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
        thread::sleep(Duration::from_micros(100));
    }
}