use horae::Utc;
use nabu::{Object, XffValue};

use crate::{
    error::LasaResult,
    parser::{Session, SessionEnd},
    utils::{new_month, new_year, probe_last_reboot},
};

pub fn construct_full_database() -> LasaResult<Object> {
    // At least one session guaranteed
    let sessions = probe_last_reboot()?;
    let mut metadata = Object::new();
    let first_boot = sessions.last().unwrap().boot_start;
    metadata.insert(
        "first_recorded_boot",
        XffValue::from_unix_timestamp(first_boot.unix_timestamp()),
    );
    let last_boot = sessions[0].boot_start;
    metadata.insert(
        "last_recorded_boot",
        XffValue::from_unix_timestamp(last_boot.unix_timestamp()),
    );

    let history = make_history_object(&sessions);

    let statistics = make_stats_object();

    let mut out = Object::new();
    out.insert("metadata", XffValue::from(metadata));
    out.insert("history", XffValue::from(history));
    out.insert("statistics", XffValue::from(statistics));

    Ok(out)
}

#[allow(clippy::too_many_lines)]
fn make_history_object(sessions: &[Session]) -> Object {
    let mut history: Object = Object::new();

    // month_number = 0 for new construction
    let (mut month, mut month_number): (Object, u8) = new_month();

    // year_number = 0 for new construction
    let (mut year, mut year_number): (Object, u16) = new_year();

    // Downtime happens BETWEEN sessions.
    // pair[0] is the later session (up_at), pair[1] is the earlier session (down_at).
    for pair in sessions.windows(2) {
        let current = &pair[0];
        let previous = &pair[1];

        let (date_time_down, event_type, down_duration) = {
            match previous.session_end {
                SessionEnd::StillRunning => unreachable!("Only the first session can be still running"),
                SessionEnd::Crash => (current.boot_start, "crash", std::time::Duration::ZERO),
                SessionEnd::Shutdown(utc_timestamp) => {
                    let duration = if current.boot_start.unix_timestamp() > utc_timestamp.unix_timestamp() {
                        current.boot_start - utc_timestamp
                    } else {
                        std::time::Duration::ZERO
                    };
                    (utc_timestamp, "reboot", duration)
                }
                SessionEnd::Recovered(utc_timestamp) => {
                    let duration = if current.boot_start.unix_timestamp() > utc_timestamp.unix_timestamp() {
                        current.boot_start - utc_timestamp
                    } else {
                        std::time::Duration::ZERO
                    };
                    (utc_timestamp, "crash", duration)
                }
            }
        };

        let down_duration: std::time::Duration = down_duration;

        if year_number == 0 {
            year_number = date_time_down.date().year;
        } else if year_number != date_time_down.date().year {
            history.insert(date_time_down.date().year.to_string(), XffValue::from(year));
            (year, _) = new_year();
            year_number = date_time_down.date().year;
        }

        if month_number == 0 {
            month_number = date_time_down.date().month;
        } else if month_number != date_time_down.date().month {
            year.insert(
                date_time_down.date().month.to_string(),
                XffValue::from(month),
            );
            (month, _) = new_month();
            month_number = date_time_down.date().month;
        }

        let mut event = Object::new();
        event.insert(
            "down_at",
            XffValue::from_unix_timestamp(date_time_down.unix_timestamp()),
        );
        event.insert(
            "up_at",
            XffValue::from_unix_timestamp(current.boot_start.unix_timestamp()),
        );
        event.insert(
            "down_duration_sec",
            XffValue::from(down_duration.as_secs_f64()),
        );
        event.insert("type", XffValue::from(event_type));

        let new_sum_monthly = month
            .get("monthly_sum_seconds")
            .unwrap()
            .into_number()
            .unwrap()
            .into_f64()
            .unwrap()
            + down_duration.as_secs_f64();

        let new_sum_yearly = year
            .get("yearly_sum_seconds")
            .unwrap()
            .into_number()
            .unwrap()
            .into_f64()
            .unwrap()
            + down_duration.as_secs_f64();

        month
            .get_mut("events")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(XffValue::from(event));
        month.insert(
            "monthly_sum_seconds",
            XffValue::from(new_sum_monthly),
        );
        year.insert(
            "yearly_sum_seconds",
            XffValue::from(new_sum_yearly),
        );
    }

    if month_number != 0 {
        if year_number == 0 {
            year_number = Utc::now().date().year;
        }
        year.insert(month_number.to_string(), XffValue::from(month));
    }

    if year_number != 0 {
        history.insert(year_number.to_string(), XffValue::from(year));
    }

    history
}

fn make_stats_object() -> Object {
    let mut all_time_stats = Object::new();
    all_time_stats.insert("uptime_percent", XffValue::from(0.0));
    all_time_stats.insert(
        "total_downtime_seconds",
        XffValue::from(0.0),
    );
    let mut current_year_stats = Object::new();
    current_year_stats.insert("year", XffValue::from(0));
    current_year_stats.insert("uptime_percent", XffValue::from(0.0));
    current_year_stats.insert(
        "total_downtime_seconds",
        XffValue::from(0.0),
    );
    let mut current_month_stats = Object::new();
    current_month_stats.insert("month", XffValue::from(0));
    current_month_stats.insert("uptime_percent", XffValue::from(0.0));
    current_month_stats.insert(
        "total_downtime_seconds",
        XffValue::from(0.0),
    );

    let mut statistics = Object::new();
    statistics.insert("all_time", all_time_stats);
    statistics.insert("current_year", current_year_stats);
    statistics.insert("current_month", current_month_stats);

    statistics
}
