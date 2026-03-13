use athena::process::{SchedulerPolicy, set_scheduler};

use crate::{
    calc_stats::calculate_statistics,
    construct_db::construct_full_database,
    env::Environment,
    error::{LasaError, LasaResult},
};

mod calc_stats;
mod construct_db;
mod env;
mod error;
mod parser;
mod utils;

fn main() -> LasaResult<()> {
    // Set scheduler to BATCH while we do heavy lifting; Low priority process though
    if let Err(err) = set_scheduler(SchedulerPolicy::Batch, 19) {
        return Err(LasaError::SettingProcessPriority(err.to_string()));
    };

    let env = Environment::new()?;

    if !env.data_file_path.exists() {
        // If the data file does not exist, first run; Clean up output files if they exist
        if env.output_file_path.exists() {
            std::fs::remove_file(&env.output_file_path).unwrap();
        }
        if env.human_readable_output_path.exists() {
            std::fs::remove_file(&env.human_readable_output_path).unwrap();
        }
        let mut db_obj = construct_full_database()?;
        calculate_statistics(&mut db_obj);
        let output_data = todo!();
        let write_data = todo!();
        Ok(())
    } else {
        run(&env)?;
        Ok(())
    }
}

fn run(env: &Environment) -> LasaResult<()> {
    let mut update_db_obj = todo!();
    calculate_statistics(&mut update_db_obj);
    let output_data = todo!();
    let write_data = todo!();
    Ok(())
}
