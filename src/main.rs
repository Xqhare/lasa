use athena::process::{SchedulerPolicy, set_scheduler};
use std::process::Command;

use crate::{
    env::Environment,
    error::{LasaError, LasaResult},
    parser::{Parser, Session, SessionEnd},
};

mod env;
mod error;
mod parser;

fn main() -> LasaResult<()> {
    let env = Environment::new()?;
    
    // Set priority to BATCH while we do the heavy lifting
    if let Err(err) = set_scheduler(SchedulerPolicy::Batch, 19) {
        eprintln!("Warning: Could not set process priority: {}", err);
    };

    run(&env)?;

    Ok(())
}

fn run(env: &Environment) -> LasaResult<()> {
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
        return Ok(());
    }

    // Handle crashes and normalization
    for (i, session) in sessions.iter_mut().enumerate() {
        let offset = i as i32; // This is a simplification; we need to calculate true boot offset
        
        if let SessionEnd::Crash = session.session_end {
            // Probe journal for last breath
            if let Ok((timestamp, offset_str)) = probe_journal(-(offset + 1)) {
                session.session_end = SessionEnd::Shutdown(Parser::normalize_to_utc(timestamp, &offset_str));
            }
        }
    }

    // TODO: Update database with new downtimes
    // Downtime = sessions[i].boot_start - sessions[i+1].session_end

    Ok(())
}

fn probe_journal(offset: i32) -> LasaResult<(horae::Utc, String)> {
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
    let line = output_str.lines().next().ok_or(LasaError::Parsing("Empty journal output".to_string()))?;
    
    // Example: 2026-03-12T07:52:10+01:00 host ...
    let iso_part = line.split_whitespace().next().ok_or(LasaError::Parsing("Invalid journal format".to_string()))?;
    
    // Split into date/time and offset
    // 2026-03-12T07:52:10+01:00
    // 0123456789012345678
    let year = iso_part[0..4].parse::<u16>().map_err(|e| LasaError::Parsing(e.to_string()))?;
    let month = iso_part[5..7].parse::<u8>().map_err(|e| LasaError::Parsing(e.to_string()))?;
    let day = iso_part[8..10].parse::<u8>().map_err(|e| LasaError::Parsing(e.to_string()))?;
    let hour = iso_part[11..13].parse::<u8>().map_err(|e| LasaError::Parsing(e.to_string()))?;
    let min = iso_part[14..16].parse::<u8>().map_err(|e| LasaError::Parsing(e.to_string()))?;
    let sec = iso_part[17..19].parse::<u8>().map_err(|e| LasaError::Parsing(e.to_string()))?;
    
    let offset_str = &iso_part[19..]; // "+01:00"
    
    let local = horae::Utc::from_ymd_hms(year, month, day, hour, min, sec);
    Ok((local, offset_str.to_string()))
}
