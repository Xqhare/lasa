use std::process::Command;

use crate::{
    error::{LasaError, LasaResult},
    parser::Parser,
};

pub fn probe_journal(offset: i32) -> LasaResult<horae::Utc> {
    let output = Command::new("journalctl")
        .arg("-b")
        .arg(offset.to_string())
        .arg("-n")
        .arg("1")
        .arg("--output=short-iso")
        .arg("--no-pager")
        .output()
        .map_err(|e| LasaError::CommandExecution(e.to_string()))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let line = output_str
        .lines()
        .next()
        .ok_or(LasaError::Parsing("Empty journal output".to_string()))?;

    // Example: 2026-03-12T07:52:10+01:00 host ...
    let iso_part = line
        .split_whitespace()
        .next()
        .ok_or(LasaError::Parsing("Invalid journal format".to_string()))?;

    // Split into date/time and offset
    // 2026-03-12T07:52:10+01:00
    let year = iso_part[0..4]
        .parse::<u16>()
        .map_err(|e| LasaError::Parsing(e.to_string()))?;
    let month = iso_part[5..7]
        .parse::<u8>()
        .map_err(|e| LasaError::Parsing(e.to_string()))?;
    let day = iso_part[8..10]
        .parse::<u8>()
        .map_err(|e| LasaError::Parsing(e.to_string()))?;
    let hour = iso_part[11..13]
        .parse::<u8>()
        .map_err(|e| LasaError::Parsing(e.to_string()))?;
    let min = iso_part[14..16]
        .parse::<u8>()
        .map_err(|e| LasaError::Parsing(e.to_string()))?;
    let sec = iso_part[17..19]
        .parse::<u8>()
        .map_err(|e| LasaError::Parsing(e.to_string()))?;

    let offset_str = &iso_part[19..]; // "+01:00"
    let offset_f64 = Parser::parse_offset_string(offset_str);

    Ok(horae::Utc::from_ymd_hms_offset(
        year, month, day, hour, min, sec, offset_f64,
    ))
}
