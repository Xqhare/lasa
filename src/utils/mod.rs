use std::process::Command;

use nabu::{Array, Object, XffValue};

use crate::{
    error::{LasaError, LasaResult},
    parser::{Parser, Session, SessionEnd},
};

/// Probe journalctl for last reboot
///
/// Returns a vector of sessions
///
/// If no sessions are found, an error is returned.
/// This means the output vector is guaranteed to have at least one element
pub fn probe_last_reboot() -> LasaResult<Vec<Session>> {
    // 1. Get last reboot output
    let output = Command::new("last")
        .arg("reboot")
        .arg("-F")
        .output()
        .map_err(|e| LasaError::CommandExecution(e.to_string()))?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let mut sessions = Parser::parse_last_reboot(&output_str);

    // 2. Identify the current boot and previous sessions
    // Sessions are returned in reverse chronological order (newest first)
    if sessions.is_empty() {
        return Err(LasaError::NoData("last reboot output is empty".to_string()));
    }

    // Handle crashes and normalization
    for (i, session) in sessions.iter_mut().enumerate() {
        let offset = i as i32;
        if let SessionEnd::Crash = session.session_end {
            // Probe journal for last breath
            if let Ok(utc_timestamp) = probe_journal(-(offset + 1)) {
                session.session_end = SessionEnd::Recovered(utc_timestamp);
            }
            // If probe fails, we just keep it as a Crash; we can't do better than that
        }
    }

    Ok(sessions)
}

pub fn new_year() -> (Object, u16) {
    let mut year = Object::new();
    year.insert("months", XffValue::from(Object::new()));
    year.insert("yearly_sum_seconds", XffValue::from(0.0));
    (year, 0)
}

pub fn new_month() -> (Object, u8) {
    let mut month = Object::new();
    month.insert("events", XffValue::from(Array::new()));
    month.insert("montly_sum_seconds", XffValue::from(0.0));
    (month, 0)
}

fn probe_journal(offset: i32) -> LasaResult<horae::Utc> {
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
