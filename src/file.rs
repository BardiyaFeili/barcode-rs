use std::{
    error::Error,
    fs::{self, File},
    io::Write,
    path::Path,
};

pub fn open_file(file: &str) -> Result<String, Box<dyn Error>> {
    let path = Path::new(file);
    if !path.exists() {
        // Return empty string for new files
        return Ok(String::new());
    }

    let contents = fs::read_to_string(file)?;
    Ok(contents)
}

pub fn save_file(file: &str, content: &[String]) -> Result<(), Box<dyn Error>> {
    let path = Path::new(file);

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent()
        && !parent.exists()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let mut f = File::create(file)?;
    for line in content {
        writeln!(f, "{}", line)?;
    }
    Ok(())
}

pub fn parent_exists(path: &str) -> bool {
    let p = Path::new(path);
    if let Some(parent) = p.parent() {
        if parent.as_os_str().is_empty() {
            return true;
        }
        parent.exists()
    } else {
        true
    }
}
