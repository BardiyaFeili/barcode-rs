use std::{
    error::Error,
    fs::{self, File, OpenOptions},
    io::Write,
};

pub fn open_file(file: &str) -> Result<String, Box<dyn Error>> {
    OpenOptions::new()
        .write(true)
        .read(true)
        .create(true)
        .truncate(false)
        .open(file)?;

    let contents = fs::read_to_string(file)?;
    Ok(contents)
}

pub fn save_file(file: &str, content: &[String]) -> Result<(), Box<dyn Error>> {
    let mut f = File::create(file)?;
    for line in content {
        writeln!(f, "{}", line)?;
    }
    Ok(())
}
