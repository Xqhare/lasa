use horae::Utc;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Session {
    pub boot_start: Utc,
    pub session_end: SessionEnd,
}

#[derive(Debug, Clone)]
pub enum SessionEnd {
    StillRunning,
    Crash,
    Shutdown(Utc),
}

pub struct Parser;

impl Parser {
    /// Parses the output of `last reboot -F`
    pub fn parse_last_reboot(output: &str) -> Vec<Session> {
        let mut sessions = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() || line.starts_with("wtmp begins") {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 10 {
                continue;
            }

            // last reboot -F format:
            // reboot system boot 6.12.73+deb13-am Wed Mar 11 17:31:18 2026 - still running
            // 0      1      2    3                4   5   6  7        8    9 10    11
            
            let start_month = Self::month_to_u8(parts[5]);
            let start_day = parts[6].parse::<u8>().unwrap_or(1);
            let start_year = parts[8].parse::<u16>().unwrap_or(1970);
            
            let time_parts: Vec<&str> = parts[7].split(':').collect();
            let start_hour = time_parts.get(0).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
            let start_min = time_parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
            let start_sec = time_parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

            let boot_start = Utc::from_ymd_hms(start_year, start_month, start_day, start_hour, start_min, start_sec);

            // Determine session end
            let session_end = if line.contains("still running") {
                SessionEnd::StillRunning
            } else if line.contains("crash") {
                SessionEnd::Crash
            } else {
                // Find the "-" separator and parse the date after it
                if let Some(dash_idx) = parts.iter().position(|&p| p == "-") {
                    if parts.len() > dash_idx + 5 {
                        let end_month = Self::month_to_u8(parts[dash_idx + 2]);
                        let end_day = parts[dash_idx + 3].parse::<u8>().unwrap_or(1);
                        let end_year = parts[dash_idx + 5].parse::<u16>().unwrap_or(1970);
                        
                        let end_time_parts: Vec<&str> = parts[dash_idx + 4].split(':').collect();
                        let end_hour = end_time_parts.get(0).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                        let end_min = end_time_parts.get(1).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);
                        let end_sec = end_time_parts.get(2).and_then(|s| s.parse::<u8>().ok()).unwrap_or(0);

                        SessionEnd::Shutdown(Utc::from_ymd_hms(end_year, end_month, end_day, end_hour, end_min, end_sec))
                    } else {
                        SessionEnd::Crash // Fallback if format is weird
                    }
                } else {
                    SessionEnd::Crash
                }
            };

            sessions.push(Session {
                boot_start,
                session_end,
            });
        }

        sessions
    }

    /// Normalizes a local timestamp to UTC given an ISO-8601 offset string like "+01:00"
    pub fn normalize_to_utc(local: Utc, offset_str: &str) -> Utc {
        // offset_str example: "+01:00" or "-05:00"
        let sign = if offset_str.starts_with('-') { -1 } else { 1 };
        let parts: Vec<&str> = offset_str[1..].split(':').collect();
        
        let hours = parts.get(0).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        let mins = parts.get(1).and_then(|s| s.parse::<i64>().ok()).unwrap_or(0);
        
        let offset_seconds = sign * (hours * 3600 + mins * 60);
        
        if sign == 1 {
            local - Duration::from_secs(offset_seconds as u64)
        } else {
            local + Duration::from_secs(offset_seconds.abs() as u64)
        }
    }

    fn month_to_u8(month: &str) -> u8 {
        match month {
            "Jan" => 1, "Feb" => 2, "Mar" => 3, "Apr" => 4,
            "May" => 5, "Jun" => 6, "Jul" => 7, "Aug" => 8,
            "Sep" => 9, "Oct" => 10, "Nov" => 11, "Dec" => 12,
            _ => 1,
        }
    }
}
