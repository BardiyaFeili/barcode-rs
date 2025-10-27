use std::{error::Error, fs::{self, OpenOptions}};

pub fn open_file(file: &String) -> Result<String, Box<dyn Error>> {
    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(&file)?;

    let contents = fs::read_to_string(&file)?;
    Ok(contents)
}
