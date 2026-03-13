use std::time::Duration;

use horae::Utc;
use nabu::{Object, XffValue};

/// Update the statistics in the database in place.
///
/// Returns a reference to the statistics object
pub fn calculate_statistics(db_obj: &mut Object) -> &Object {
    let first_recorded_boot = get_first_boot(db_obj);
    let (all_time_sum, yearly_sum, montly_sum, today) = get_sums(db_obj);

    let all_time_dur = today - first_recorded_boot;
    let all_time_up_perc =
        (1.0 - (all_time_sum.as_secs_f64() / all_time_dur.as_secs_f64())) * 100.0;

    let this_year_dur = today - Utc::from_ymd_hms(today.date().year, 1, 1, 0, 0, 0);
    let this_year_up_perc =
        (1.0 - (yearly_sum.as_secs_f64() / this_year_dur.as_secs_f64())) * 100.0;

    let this_month_dur =
        today - Utc::from_ymd_hms(today.date().year, today.date().month, 1, 0, 0, 0);
    let this_month_up_perc =
        (1.0 - (montly_sum.as_secs_f64() / this_month_dur.as_secs_f64())) * 100.0;

    let mut stats = db_obj
        .get_mut("statistics")
        .unwrap()
        .as_object_mut()
        .unwrap();
    let all_time = stats.get_mut("all_time").unwrap().as_object_mut().unwrap();
    all_time.insert("uptime_percent", XffValue::from(all_time_up_perc));
    all_time.insert(
        "total_downtime_seconds",
        XffValue::from(all_time_sum.as_secs_f64()),
    );
    let current_year = stats
        .get_mut("current_year")
        .unwrap()
        .as_object_mut()
        .unwrap();
    current_year.insert("year", XffValue::from(today.date().year));
    current_year.insert("uptime_percent", XffValue::from(this_year_up_perc));
    current_year.insert(
        "total_downtime_seconds",
        XffValue::from(yearly_sum.as_secs_f64()),
    );
    let current_month = stats
        .get_mut("current_month")
        .unwrap()
        .as_object_mut()
        .unwrap();
    current_month.insert("month", XffValue::from(today.date().month));
    current_month.insert("uptime_percent", XffValue::from(this_month_up_perc));
    current_month.insert(
        "total_downtime_seconds",
        XffValue::from(montly_sum.as_secs_f64()),
    );
    let stats_ptr = db_obj.get("statistics").unwrap().as_object().unwrap();
    stats_ptr
}

/// 1x 2x O(1) lookups
fn get_first_boot(db_obj: &Object) -> Utc {
    Utc::from_timestamp(
        db_obj
            .get("metadata")
            .unwrap()
            .as_object()
            .unwrap()
            .get("first_recorded_boot")
            .unwrap()
            .into_unix_timestamp()
            .unwrap(),
    )
}

/// 3x 1x O(1) lookups
fn get_sums(db_obj: &Object) -> (Duration, Duration, Duration, Utc) {
    let (today, current_year, current_month) = {
        let mut today = Utc::now();
        today.with_auto_offset();
        (today, today.date().year, today.date().month)
    };
    let (all_time_sum, yearly_sum, montly_sum) = {
        if let Some(history) = db_obj.get("history") {
            let history = history.as_object().unwrap();
            let all_time_sum = construct_all_time_sum(history);
            if let Some(year) = history.get(&current_year.to_string()) {
                let year = year.as_object().unwrap();
                let yearly_sum = year
                    .get("yearly_sum_seconds")
                    .unwrap()
                    .into_std_duration()
                    .unwrap();

                if let Some(month) = year.get(&current_month.to_string()) {
                    let montly_sum = month
                        .as_object()
                        .unwrap()
                        .get("montly_sum_seconds")
                        .unwrap()
                        .into_std_duration()
                        .unwrap();
                    (all_time_sum, yearly_sum, montly_sum)
                } else {
                    // Today's month has no data - new month with no boot yet or something went terribly wrong
                    // Assumed edge case - just set to 0 seconds.
                    // If a reboot happens, month will be picked up by the next run
                    (all_time_sum, yearly_sum, Duration::ZERO)
                }
            } else {
                // Today's year (and thus month) has no data - new year with no boot yet or something went terribly wrong
                // Assumed edge case - just set to 0 seconds.
                // If a reboot happens, year and month will be picked up by the next run
                (all_time_sum, Duration::ZERO, Duration::ZERO)
            }
        } else {
            // No history??
            debug_assert!(false, "No history - should be unreachable");
            (Duration::ZERO, Duration::ZERO, Duration::ZERO)
        }
    };

    (all_time_sum, yearly_sum, montly_sum, today)
}

/// 1x O(n) where n is the number of years
fn construct_all_time_sum(history: &Object) -> Duration {
    let mut sum_dur = Duration::ZERO;
    for (_key, value) in history.iter() {
        if let Some(year_obj) = value.as_object() {
            if let Some(sum) = year_obj.get("yearly_sum_seconds") {
                sum_dur += sum.into_std_duration().unwrap();
            }
        }
    }
    sum_dur
}
