use crate::utils::{
    driver::{close_driver, launch_driver},
    utils::{calculate_progress_weight, calculate_split, find_available_port, get_download_dir, unzip_file, wait_until_file_downloaded},
};

use chrono::{Datelike, DateTime, Utc};
use futures::future::join_all;
use polars::prelude::*;
use std::{
    fs::{File, remove_dir_all},
    path::PathBuf,
    sync::Arc,
};
use thirtyfour::prelude::*;
use tokio::sync::mpsc::Sender;

pub async fn download_data(pair: String, from_date: DateTime<Utc>, to_date: DateTime<Utc>, data_dir: PathBuf, data_type: String, tx: Sender<usize>) -> Result<(), String> {
    // Calculate the number of years to download
    let from_year = from_date.year() as usize;
    let to_year = to_date.year() as usize;
    let year_duration = to_year - from_year + 1 as usize;

    // Then we split it to make it parallel
    let split = calculate_split(from_year, year_duration);

    // Calculate the progress weight
    // So we have a good display of the progress
    let progress_weight = Arc::new(calculate_progress_weight(year_duration));

    // Get the default download directory
    let download_dir = get_download_dir().unwrap();

    // Init the main DataFrame
    let mut main_df = DataFrame::new(vec![
        Column::new("datetime".into(), Vec::<i64>::new()).cast(&DataType::Datetime(TimeUnit::Microseconds, None)).unwrap(),
        Column::new("open".into(), Vec::<f64>::new()),
        Column::new("high".into(), Vec::<f64>::new()),
        Column::new("low".into(), Vec::<f64>::new()),
        Column::new("close".into(), Vec::<f64>::new()),
        Column::new("volume".into(), Vec::<i64>::new()),
    ]).unwrap();
    
    // Create arc to permit sharing the data between threads
    let pair = Arc::new(pair);
    let download_dir = Arc::new(download_dir);
    let tx = Arc::new(tx);

    // Create the different tasks and spawn them
    // Store the results in tasks
    let mut tasks = Vec::new();
    for repartition in split {
        let pair = Arc::clone(&pair);
        let download_dir = Arc::clone(&download_dir);
        let tx = Arc::clone(&tx);
        let progress_weight = Arc::clone(&progress_weight);

        tasks.push(tokio::spawn(async move {
            download_split_data(pair, download_dir, repartition, tx, progress_weight).await
        }));
    }

    // Wait for all the tasks to finish
    let results = join_all(tasks).await;

    // Check for errors and if not merge the DataFrames
    for result in results {
        match result {
            Ok(Ok(df)) => {
                main_df.vstack_mut(&df).unwrap();
            }
            Ok(Err(e)) => {
                eprintln!("Error downloading data: {}", e);
            }
            Err(e) => {
                eprintln!("Error spawning task: {}", e);
            }
        }
    }

    // Only keep the data between the dates asked
    main_df = main_df.lazy()
                     .filter(col("datetime").gt(lit(from_date.timestamp_millis() * 1000)))
                     .filter(col("datetime").lt(lit(to_date.timestamp_millis() * 1000)))
                     .collect().map_err(|_| "Failed to filter DataFrame")?;

    // Save the data
    save_data(&mut main_df, &data_dir, &pair, &data_type)?;

    // Signal the progress that we have finished
    tx.send(0).await.map_err(|_| "Failed to send progress")?;

    Ok(())
}

// This function split is used to execute the different tasks in parallel
pub async fn download_split_data(pair: Arc<String>, download_dir: Arc<String>, years: Vec<usize>, tx: Arc<Sender<usize>>, progress_weight: Arc<usize>) -> Result<DataFrame, String> {
    // Init the main DataFrame
    let mut main_df = DataFrame::new(vec![
        Column::new("datetime".into(), Vec::<i64>::new()).cast(&DataType::Datetime(TimeUnit::Microseconds, None)).unwrap(),
        Column::new("open".into(), Vec::<f64>::new()),
        Column::new("high".into(), Vec::<f64>::new()),
        Column::new("low".into(), Vec::<f64>::new()),
        Column::new("close".into(), Vec::<f64>::new()),
        Column::new("volume".into(), Vec::<i64>::new()),
    ]).unwrap();

    // Find an available port
    // And launch the driver
    let port = find_available_port(9000, 9500).await;
    launch_driver(port).await.unwrap();

    // Put some arguments to the driver
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless").map_err(|_| "Failed to add argument: --headless")?;
    caps.add_arg("--disable-gpu").map_err(|_| "Failed to add argument: --disable-gpu")?;
    caps.add_arg("--no-sandbox").map_err(|_| "Failed to add argument: --no-sandbox")?;
    caps.add_arg("--disable-dev-shm-usage").map_err(|_| "Failed to add argument: --disable-dev-shm-usage")?;

    // Use the driver
    let driver = match WebDriver::new(format!("http://localhost:{}", port), caps).await {
        Ok(d) => d,
        Err(_) => return Err("Failed to create WebDriver".into()),
    };

    for year in years {
        let lowercase_pair = pair.to_lowercase();

        // Find the download link
        // And click on it
        driver.get(format!("https://www.histdata.com/download-free-forex-historical-data/?/ascii/1-minute-bar-quotes/{}/{}", lowercase_pair, year)).await.map_err(|_| "Failed to open URL")?;
        let elem = driver.find(By::Id("a_file")).await.map_err(|_| "Failed to find element: a_file")?;
        elem.click().await.map_err(|_| "Failed to click element: a_file")?;

        // Wait for the download to finish
        let file = format!("{}/HISTDATA_COM_ASCII_{}_M1{}.zip", download_dir, pair, year);
        wait_until_file_downloaded(&file);
        match unzip_file(&file) {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to unzip file: {}", e)),
        }

        // Notify that we are done with the download
        tx.send(*progress_weight).await.map_err(|_| "Failed to send progress")?;

        // Force all the columns to be string
        let schema = Schema::from_iter(vec![
            Field::new("column_1".into(), DataType::String),
            Field::new("column_2".into(), DataType::String),
            Field::new("column_3".into(), DataType::String),
            Field::new("column_4".into(), DataType::String),
            Field::new("column_5".into(), DataType::String),
            Field::new("column_6".into(), DataType::String),
        ]);
        
        // Get the data from the file downloaded
        let file_path = format!("{}/DAT_ASCII_{}_M1_{}.csv", &file.strip_suffix(".zip").unwrap(), pair, year);
        let mut df = LazyCsvReader::new(&file_path).with_separator(b';').with_has_header(false)
            .with_schema(Some(Arc::new(schema)))
            .finish().unwrap().collect().unwrap();

        // Parse all the columns
        df = df.lazy()
            .with_columns([
                col("column_1")
                .str()
                .to_datetime(
                    Some(TimeUnit::Microseconds),
                    None,
                    StrptimeOptions {
                        format: Some("%Y%m%d %H%M%S".into()),
                        ..Default::default()
                    },
                    lit("raise"),
                )
                .alias("datetime"),
                col("column_2").cast(DataType::String).str().replace_all(lit(" "), lit(""), false).cast(DataType::Float64).alias("open"),
                col("column_3").cast(DataType::String).str().replace_all(lit(" "), lit(""), false).cast(DataType::Float64).alias("high"),
                col("column_4").cast(DataType::String).str().replace_all(lit(" "), lit(""), false).cast(DataType::Float64).alias("low"),
                col("column_5").cast(DataType::String).str().replace_all(lit(" "), lit(""), false).cast(DataType::Float64).alias("close"),
                col("column_6").cast(DataType::String).str().replace_all(lit(" "), lit(""), false).cast(DataType::Int64).alias("volume"),
            ])
        .drop([
            "column_1",
            "column_2",
            "column_3",
            "column_4",
            "column_5",
            "column_6",
        ])
        .collect().unwrap();

        // Merge the DataFrame with the main one
        main_df = main_df.vstack(&df).unwrap();

        // Remove the downloaded file
        remove_dir_all(&file.strip_suffix(".zip").unwrap()).map_err(|_| "Failed to remove directory")?;

        // Notify that we are done with the parsing
        tx.send(*progress_weight).await.map_err(|_| "Failed to send progress")?;
    }

    // Close the driver and the server
    driver.quit().await.map_err(|_| "Failed to quit driver")?;
    close_driver(port).await.map_err(|_| "Failed to close browser")?;

    Ok(main_df)
}

pub fn save_data(df: &mut DataFrame, data_dir: &PathBuf, pair: &str, data_type: &str) -> Result<(), String> {
    // Get the paths
    let file_path = format!("{}/{}.{}", data_dir.display(), pair, data_type);
    let file = File::create(&file_path).map_err(|e| format!("Failed to create file {}: {}", file_path, e))?;

    // Save the data with the right format
    match data_type {
        "csv" => {
            CsvWriter::new(file)
                .finish(df)
                .map_err(|e| format!("Failed to write CSV file: {}", e))?;
        }
        "parquet" => {
            ParquetWriter::new(file)
                .finish(df)
                .map_err(|e| format!("Failed to write Parquet file: {}", e))?;
        }
        _ => {
            return Err(format!("Unsupported data type: {}", data_type));
        }
    }
    
    Ok(())
}