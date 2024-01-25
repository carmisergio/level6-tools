use std::fs;
use std::io;
use std::io::Read;
use std::io::Write;
use std::path::PathBuf;

// Read file to Vec<u8>
pub fn read_file(file_path: &PathBuf) -> Result<Vec<u8>, io::Error> {
    // Open file
    let mut file = fs::File::open(file_path)?;

    // Read file to Vec<u8>
    let mut read_buf: Vec<u8> = Vec::new();
    file.read_to_end(&mut read_buf)?;

    Ok(read_buf)
}

// Write file from Vec<u8>
pub fn write_file(file_path: &PathBuf, data: Vec<u8>) -> Result<(), io::Error> {
    // Open file
    let mut file = fs::OpenOptions::new()
        .create(true) // Create new file if it doesn't exist
        .write(true)
        .truncate(true) // Allow overwriting
        .open(file_path)?;

    // Write file
    file.write_all(&data)?;
    file.flush()?;

    Ok(())
}
