use std::fs::{OpenOptions, File};
use std::io::{Write, BufWriter};
use std::time::{SystemTime, UNIX_EPOCH};
use std::error::Error;
use chrono::Local;

// keeps track of whether we've already cleared the log this run
static mut LOG_CLEARED: bool = false;

fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    let tm = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    format!("[{} | {}s]", tm, secs)
}

pub fn log<S: AsRef<str>>(message: S) -> Result<(), Box<dyn Error>> {
    // open in append mode after initial clear
    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("log.txt")?;
    let mut writer = BufWriter::new(file);

    writeln!(writer, "{} {}", timestamp(), message.as_ref())?;
    writer.flush()?;
    Ok(())
}
pub fn log_startup(app_name: &str, version: &str) -> Result<(), Box<dyn Error>> {
    unsafe {
        if !LOG_CLEARED {
            // truncate file at first run
            File::create("log.txt")?;
            LOG_CLEARED = true;
        }
    }

    let info = format!(
        "{} v{} started at {}\n",
        app_name,
        version,
        Local::now().format("%Y-%m-%d %H:%M:%S")
    );

    println!("{}", info);
    log(&format!("--- APP STARTUP ---\n{}", info))?;
    Ok(())
}