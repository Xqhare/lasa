use nabu::{Object, XffValue, serde::read};

use crate::{
    env::Environment,
    error::{LasaError, LasaResult},
    parser::{Session, SessionEnd},
    utils::{new_month, new_year, probe_last_reboot},
};

pub fn update_database(env: &Environment) -> LasaResult<Object> {
    let mut db = read_db(env)?;
    // At least one session guaranteed
    let sessions = probe_last_reboot()?;
    let old_last_recorded_boot = update_metadata(&mut db, &sessions[0]);
    let history = db.get_mut("history").unwrap().as_object_mut().unwrap();

    for session in sessions.iter() {
        let (date_time_down, event_type) = {
            match session.session_end {
                SessionEnd::StillRunning => continue,
                SessionEnd::Crash => unreachable!("Checked earlier"),
                SessionEnd::Shutdown(utc_timestamp) => {
                    if utc_timestamp.unix_timestamp() <= old_last_recorded_boot {
                        break;
                    }
                    (utc_timestamp, "reboot")
                }
                SessionEnd::Recovered(utc_timestamp) => {
                    if utc_timestamp.unix_timestamp() <= old_last_recorded_boot {
                        break;
                    }
                    (utc_timestamp, "crash")
                }
            }
        };
        let down_duration = session.boot_start - date_time_down;

        let year = {
            if let Some(year) = history.get_mut(&date_time_down.date().year.to_string()) {
                year.as_object_mut().unwrap()
            } else {
                let (year, _) = new_year();
                history.insert(date_time_down.date().year.to_string(), XffValue::from(year));
                history
                    .get_mut(&date_time_down.date().year.to_string())
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
            }
        };

        let new_sum_yearly = year
            .get("yearly_sum_seconds")
            .unwrap()
            .into_duration_seconds()
            .unwrap()
            + down_duration.as_secs_f64();

        let month = {
            if let Some(month) = year.get_mut(&date_time_down.date().month.to_string()) {
                month.as_object_mut().unwrap()
            } else {
                let (month, _) = new_month();
                year.insert(
                    date_time_down.date().month.to_string(),
                    XffValue::from(month),
                );
                year.get_mut(&date_time_down.date().month.to_string())
                    .unwrap()
                    .as_object_mut()
                    .unwrap()
            }
        };

        let new_sum_montly = month
            .get("montly_sum_seconds")
            .unwrap()
            .into_duration_seconds()
            .unwrap()
            + down_duration.as_secs_f64();

        let mut event: Object = Object::new();
        event.insert(
            "down_at",
            XffValue::from_unix_timestamp(date_time_down.unix_timestamp()),
        );
        event.insert(
            "up_at",
            XffValue::from_unix_timestamp(session.boot_start.unix_timestamp()),
        );
        event.insert(
            "down_duration_sec",
            XffValue::from_duration_seconds(down_duration.as_secs_f64()),
        );
        event.insert("type", XffValue::from(event_type));

        month
            .get_mut("events")
            .unwrap()
            .as_array_mut()
            .unwrap()
            .push(XffValue::from(event));

        month.insert(
            "montly_sum_seconds",
            XffValue::from_duration_seconds(new_sum_montly),
        );
        year.insert(
            "yearly_sum_seconds",
            XffValue::from_duration_seconds(new_sum_yearly),
        );
    }

    Ok(db)
}

fn update_metadata(db: &mut Object, first_session: &Session) -> f64 {
    let metadata = db.get_mut("metadata").unwrap().as_object_mut().unwrap();
    let old_last_recorded_boot = metadata
        .get("last_recorded_boot")
        .unwrap()
        .into_unix_timestamp()
        .unwrap();
    metadata.insert(
        "last_recorded_boot",
        XffValue::from_unix_timestamp(first_session.boot_start.unix_timestamp()),
    );
    old_last_recorded_boot
}

fn read_db(env: &Environment) -> LasaResult<Object> {
    match read(&env.data_file_path) {
        Ok(db) => Ok(db.into_object().unwrap()),
        Err(e) => Err(LasaError::DataStorage(e.to_string())),
    }
}
