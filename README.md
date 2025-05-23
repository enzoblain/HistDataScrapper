# 🚀 HistDataScrapper

HistDataScrapper is a lively and easy-to-use command-line tool to grab 1-minute candlestick data for forex pairs straight from HistData.com. Powered by ChromeDriver automation and written in Rust, it leverages the language’s strong concurrency model to efficiently handle multiple downloads in parallel. 🦀⚡

Thanks to Rust’s safety and performance, HistDataScrapper provides a fast and reliable experience. The interactive terminal interface makes selecting currency pairs, dates, and file formats a breeze. 🎯📈

## ⚙️ Prerequisites

- **ChromeDriver** must be installed and available in your system PATH.  
  Download it here: https://chromedriver.chromium.org/downloads 🔗

- Get the latest release of HistDataScrapper from the GitHub repo:  
  https://github.com/enzoblain/HistDataScrapper/releases 📦

## ▶️ How to run

Just download and run the executable — no need for manual building if you use the releases.

```
./HistDataScrapper
```

## 🎛️ Interface walkthrough

1. **Select a currency pair**  
   Pick from a curated list of forex pairs like EUR/USD, USD/JPY, GBP/USD, and more using your keyboard arrows. ⬆️⬇️

2. **Enter beginning date**  
   Type the start date (`YYYY-MM-DD`), which must be within the available data range (from the pair’s minimum date up to Dec 31, 2024). The prompt will keep asking until a valid date is entered. 📅

3. **Enter end date**  
   Choose an end date (`YYYY-MM-DD`) between your start date and Dec 31, 2024. Invalid inputs will prompt re-entry. ⏳

4. **Choose destination folder**  
   Specify where you want the data saved. If the folder doesn’t exist, the program will create it for you. 📂

5. **Choose data format**  
   Select whether you want CSV or Parquet for your saved data. 💾

6. **Watch download progress**  
   A sleek progress bar updates in real-time while your data downloads. ⬇️📊

7. **Completion message**  
   Once done, you’ll see a confirmation with the file location and name. ✅🎉

## 📝 Example usage

```
$ ./HistDataScrapper
Select a currency pair: AUDCAD
> Enter beginning date (YYYY-MM-DD), between 2007-01-01 and 2024-12-31:  2007-01-01
> Enter end date (YYYY-MM-DD), between 2007-01-01 and 2024-12-31:        2024-12-31
> Where do you want to save the data? data
Select a data type: csv
[00:00:05] [████▏.................................]   8% (15s)
Data downloaded successfully!
```

## 🚨 macOS Gatekeeper Notice 🚨

If you see this warning:

❗ *"Apple cannot check `histdatascraper-macos` for malicious software."*

It means macOS flagged the app with a **quarantine** attribute. To fix it, just run this command in your terminal:

```
xattr -d com.apple.quarantine /path/to/histdatascraper-macos
```

For example:

```
xattr -d com.apple.quarantine /Users/me/Downloads/histdatascraper-macos
```

✅ This removes the quarantine flag and lets you open the app normally!

## 📌 Notes

- Make sure your ChromeDriver version matches your Chrome browser. 🔄  
- The tool runs asynchronously and can be stopped anytime by closing it. ⏹️  
- Data files are saved as `<PAIR>.<EXT>` (e.g., `EURUSD.csv` or `EURUSD.parquet`) in your chosen directory. 💼

---

**Author:** Enzo Blain  
**GitHub:** https://github.com/enzoblain/HistDataScrapper 🌟