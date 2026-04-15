use std::io::{BufRead, BufReader, Read};

pub struct PpmImage {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl PpmImage {
    pub fn try_from(value: &[u8]) -> Result<Self, String> {
        let mut reader = BufReader::new(value);
        let mut line = String::new();
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read line: {}", e))?;
        if line.trim() != "P6" {
            return Err("Not a valid PPM file (expected P6)".to_string());
        }
        line.clear();
        let (width, height) = {
            reader
                .read_line(&mut line)
                .map_err(|e| format!("Failed to read line: {}", e))?;
            let trimmed = line.trim();
            let parts: Vec<&str> = trimmed.split_whitespace().collect();
            (
                parts[0].parse().map_err(|_| "Invalid width")?,
                parts[1].parse().map_err(|_| "Invalid height")?,
            )
        };
        reader
            .read_line(&mut line)
            .map_err(|e| format!("Failed to read max value line: {}", e))?;
        let mut raw_data = vec![0u8; (width * height * 3) as usize];
        reader
            .read_exact(&mut raw_data)
            .map_err(|e| format!("Failed to read image data: {}", e))?;
        // Force output to RGBA by setting alpha channel to full opacity
        let mut data = Vec::with_capacity(raw_data.len() / 3 * 4);
        for chunk in raw_data.chunks(3) {
            data.push(chunk[0]);
            data.push(chunk[1]);
            data.push(chunk[2]);
            data.push(255);
        }
        Ok(PpmImage {
            width,
            height,
            data,
        })
    }
}
