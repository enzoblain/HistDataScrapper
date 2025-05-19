use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync:: Mutex;

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