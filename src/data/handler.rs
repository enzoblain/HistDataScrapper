use crate::PORT;
use crate::utils::utils::{get_download_dir, unzip_file, wait_until_file_downloaded};

use chrono::{Datelike, DateTime, NaiveDateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use polars::prelude::*;
use std::{collections::HashMap, fs::File, path::PathBuf, sync::Arc};
use thirtyfour::prelude::*;
use tokio::sync::{mpsc::Sender, Mutex};

pub static PAIRS: Lazy<Mutex<HashMap<String, DateTime<Utc>>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub async fn build_pairs() {
    // Create a new HashMap to store the pairs and their start dates
    macro_rules! insert {
        ($pair:literal, $year:literal) => {
            let key = $pair.replace("/", "");
            // Generate the date from the year
            let dt = NaiveDateTime::parse_from_str(
                &format!("{}-01-01 00:00:00", $year),
                "%Y-%m-%d %H:%M:%S"
            )
            .map(|dt| Utc.from_utc_datetime(&dt))
            .unwrap();

            // Insert the pair and its start date into the HashMap
            {
                let mut pairs_lock = PAIRS.lock().await;
                pairs_lock.insert(key.clone(), dt);
            };
        };
    }

    // All pairs from histdata.com
    insert!("AUDCAD", 2007);
    insert!("AUDCHF", 2008);
    insert!("AUDJPY", 2002);
    insert!("AUDNZD", 2007);
    insert!("AUDUSD", 2000);
    insert!("AUXAUD", 2010);
    insert!("BCOUSD", 2010);
    insert!("CADCHF", 2008);
    insert!("CADJPY", 2007);
    insert!("CHFJPY", 2002);
    insert!("ETXEUR", 2010);
    insert!("EURAUD", 2002);
    insert!("EURCAD", 2007);
    insert!("EURCHF", 2000);
    insert!("EURCZK", 2010);
    insert!("EURDKK", 2008);
    insert!("EURGBP", 2002);
    insert!("EURHUF", 2010);
    insert!("EURJPY", 2002);
    insert!("EURNOK", 2008);
    insert!("EURNZD", 2008);
    insert!("EURPLN", 2010);
    insert!("EURSEK", 2008);
    insert!("EURTRY", 2010);
    insert!("EURUSD", 2000);
    insert!("FRXEUR", 2010);
    insert!("GBPCHF", 2010);
    insert!("GBPCAD", 2007);
    insert!("GBPJPY", 2002);
    insert!("GBPNZD", 2008);
    insert!("GBPAUD", 2007);
    insert!("GBPUSD", 2000);
    insert!("GRXEUR", 2010);
    insert!("HKXHKD", 2010);
    insert!("JPXJPY", 2010);
    insert!("NSXUSD", 2010);
    insert!("NZDCAD", 2008);
    insert!("NZDCHF", 2008);
    insert!("NZDJPY", 2006);
    insert!("NZDUSD", 2005);
    insert!("SGDJPY", 2008);
    insert!("SPXUSD", 2010);
    insert!("UDXUSD", 2010);
    insert!("UKXGBP", 2010);
    insert!("USDCAD", 2002);
    insert!("USDCHF", 2000);
    insert!("USDCZK", 2010);
    insert!("USDDKK", 2008);
    insert!("USDHKD", 2008);
    insert!("USDHUF", 2010);
    insert!("USDJPY", 2000);
    insert!("USDMXN", 2000);
    insert!("USDNOK", 2008);
    insert!("USDPLN", 2010);
    insert!("USDSGD", 2008);
    insert!("USDSEK", 2008);
    insert!("USDTRY", 2010);
    insert!("USDZAR", 2010);
    insert!("WTIUSD", 2010);
    insert!("XAUAUD", 2009);
    insert!("XAUCHF", 2009);
    insert!("XAUEUR", 2009);
    insert!("XAUGBP", 2009);
    insert!("XAUUSD", 2009);
    insert!("XAGUSD", 2009);
    insert!("ZARJPY", 2010);
}


pub async fn downloader(tx: Sender<u64>, pair: String, from: DateTime<Utc>, to: DateTime<Utc>, destination: PathBuf, datatype: String) -> Result<(), String> {
    // Create a DataFrame to store the data
    let mut main_df = DataFrame::new(vec![
        Column::new("datetime".into(), Vec::<i64>::new()).cast(&DataType::Datetime(TimeUnit::Microseconds, None)).unwrap(),
        Column::new("open".into(), Vec::<f64>::new()),
        Column::new("high".into(), Vec::<f64>::new()),
        Column::new("low".into(), Vec::<f64>::new()),
        Column::new("close".into(), Vec::<f64>::new()),
        Column::new("volume".into(), Vec::<i64>::new()),
    ]).unwrap();

    // Get the available port
    let port = {
        let port_lock = PORT.lock().await;
        *port_lock
    };

    // Launch the browser
    // And do not show it
    let mut caps = DesiredCapabilities::chrome();
    caps.add_arg("--headless").map_err(|_| "Failed to add argument: --headless")?;
    let driver = match WebDriver::new(format!("http://localhost:{}", port), caps).await {
        Ok(d) => d,
        Err(_) => return Err("Failed to create WebDriver".into()),
    };

    // Find the download directory of the user
    let download_dir = get_download_dir().unwrap();

    // Get the beginning and ending year from the date range
    let beginning_year = from.year();
    let ending_year = to.year();

    let lowercase_pair = pair.to_lowercase();

    // Iterate through the years and download the data
    for year in beginning_year..ending_year + 1 {
        // Connect to the website
        // And download the data
        driver.get(format!("https://www.histdata.com/download-free-forex-historical-data/?/ascii/1-minute-bar-quotes/{}/{}", lowercase_pair, year)).await.map_err(|_| "Failed to open URL")?;
        let elem = driver.find(By::Id("a_file")).await.map_err(|_| "Failed to find element: a_file")?;
        elem.click().await.map_err(|_| "Failed to click element: a_file")?;

        // Wait for the download to finish
        // And then handle the file
        let file = format!("{}/HISTDATA_COM_ASCII_{}_M1{}.zip", download_dir, pair, year);
        wait_until_file_downloaded(&file);
        match unzip_file(&file, "data") {
            Ok(_) => (),
            Err(e) => return Err(format!("Failed to unzip file: {}", e)),
        }

        tx.send(1).await.map_err(|_| "Failed to send progress")?;

        // Force all the columns to be string
        let schema = Schema::from_iter(vec![
            Field::new("column_1".into(), DataType::String),
            Field::new("column_2".into(), DataType::String),
            Field::new("column_3".into(), DataType::String),
            Field::new("column_4".into(), DataType::String),
            Field::new("column_5".into(), DataType::String),
            Field::new("column_6".into(), DataType::String),
        ]);

        // Save the file paths
        let file_path = format!("{}/DAT_ASCII_{}_M1_{}.csv", destination.to_string_lossy(), pair, year);
        let other_file_path = format!("{}/DAT_ASCII_{}_M1_{}.txt", destination.to_string_lossy(), pair, year);

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

        // If it's the first year, we have to filter the data (start where the user wants)
        if year == beginning_year {
            df = df.lazy()
                .filter(col("datetime").gt(lit(from.timestamp_millis())))
                .collect().map_err(|_| "Failed to filter DataFrame")?;
        }
        
        // If it's the last year, we have to filter the data (stop where the user wants)
        else if year == ending_year {
            df = df.lazy()
                    .filter(col("datetime").lt(lit(to.timestamp_millis())))
                    .collect().map_err(|_| "Failed to filter DataFrame")?;
        }

        main_df = main_df.vstack(&df).unwrap();

        // Remove the files
        std::fs::remove_file(&file_path).expect("Failed to remove file");
        std::fs::remove_file(&other_file_path).expect("Failed to remove file");

        tx.send(1).await.map_err(|_| "Failed to send progress")?;
    }

    match datatype.as_str() {
        "csv" => {
            // Save the DataFrame to a CSV file
            CsvWriter::new(File::create(format!("{}/{}.csv", destination.to_string_lossy().to_string(), pair)).unwrap()).finish(&mut main_df).unwrap();
        }
        "parquet" => {
            // Save the DataFrame to a Parquet file
            ParquetWriter::new(File::create(format!("{}/{}.parquet", destination.to_string_lossy().to_string(), pair)).unwrap()).finish(&mut main_df).unwrap();
        }
        _ => {}
    }

    tx.send(1).await.map_err(|_| "Failed to send progress")?;

    Ok(())
}