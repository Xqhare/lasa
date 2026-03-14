use std::path::PathBuf;

use areia::BaseDirs;

use crate::error::{LasaError, LasaResult};

#[allow(dead_code)]
/// The environment of the application
///
/// # Note
/// If the environment cannot find a runtime directory, it will use the home directory as a
/// fallback
///
/// # Fields
/// * `home_dir` - The home directory of the user
/// * `data_file_path` - The path to the data file - Inside application dir
/// * `output_file_path` - The path to the output file - Inside Home
/// * `human_readable_output_path` - The path to the human readable output file - Inside Home
/// * `heartbeat_path` - The path to the heartbeat file - Inside application dir
/// * `last_shutdown_path` - The path to the last shutdown file - Inside application dir
pub struct Environment {
    pub home_dir: PathBuf,
    pub data_file_path: PathBuf,
    pub output_file_path: PathBuf,
    pub human_readable_output_path: PathBuf,
}

impl Environment {
    pub fn new() -> LasaResult<Environment> {
        let home = match BaseDirs::new() {
            Ok(home) => home,
            Err(err) => return Err(LasaError::EnvironmentError(err.to_string())),
        };

        let home_dir = home.home_dir().to_owned();
        let data_dir = {
            if let Some(dir) = home.runtime_dir() {
                dir
            } else {
                home.home_dir()
            }
        };
        let data_file_path = data_dir.join("lasa_system_uptime_db.xff");

        let output_file_path = home_dir.join("lasa_system_uptime.xff");
        let human_readable_output_path = home_dir.join("lasa_system_uptime.json");

        Ok(Environment {
            home_dir,
            data_file_path,
            output_file_path,
            human_readable_output_path,
        })
    }
}
