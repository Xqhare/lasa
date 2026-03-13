use horae::{Utc, detect_local_utc_offset};

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
    Recovered(Utc),
}

pub struct Parser;

impl Parser {
    /// Parses the output of `last reboot -F`
    pub fn parse_last_reboot(output: &str) -> Vec<Session> {
        let mut sessions = Vec::new();
        let local_offset = detect_local_utc_offset().unwrap_or(0.0);

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
            let start_hour = time_parts
                .get(0)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);
            let start_min = time_parts
                .get(1)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);
            let start_sec = time_parts
                .get(2)
                .and_then(|s| s.parse::<u8>().ok())
                .unwrap_or(0);

            // last reboot outputs local time. Instantiate with the local offset to normalize to UTC.
            let boot_start = Utc::from_ymd_hms_offset(
                start_year,
                start_month,
                start_day,
                start_hour,
                start_min,
                start_sec,
                local_offset,
            );

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
                        let end_hour = end_time_parts
                            .get(0)
                            .and_then(|s| s.parse::<u8>().ok())
                            .unwrap_or(0);
                        let end_min = end_time_parts
                            .get(1)
                            .and_then(|s| s.parse::<u8>().ok())
                            .unwrap_or(0);
                        let end_sec = end_time_parts
                            .get(2)
                            .and_then(|s| s.parse::<u8>().ok())
                            .unwrap_or(0);

                        let shutdown_time = Utc::from_ymd_hms_offset(
                            end_year,
                            end_month,
                            end_day,
                            end_hour,
                            end_min,
                            end_sec,
                            local_offset,
                        );
                        SessionEnd::Shutdown(shutdown_time)
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

    /// Parses an ISO-8601 offset string (e.g., "+01:00") into f64 hours.
    pub fn parse_offset_string(offset_str: &str) -> f64 {
        if offset_str.len() < 6 {
            return 0.0;
        }

        let sign = if offset_str.starts_with('-') {
            -1.0
        } else {
            1.0
        };
        let hours = offset_str[1..3].parse::<f64>().unwrap_or(0.0);
        let mins = offset_str[4..6].parse::<f64>().unwrap_or(0.0);

        sign * (hours + (mins / 60.0))
    }

    fn month_to_u8(month: &str) -> u8 {
        match month {
            "Jan" => 1,
            "Feb" => 2,
            "Mar" => 3,
            "Apr" => 4,
            "May" => 5,
            "Jun" => 6,
            "Jul" => 7,
            "Aug" => 8,
            "Sep" => 9,
            "Oct" => 10,
            "Nov" => 11,
            "Dec" => 12,
            _ => 1,
        }
    }
}
