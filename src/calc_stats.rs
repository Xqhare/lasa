use nabu::serde::read;

use crate::{env::Environment, error::{LasaError, LasaResult}};

pub fn calculate_statistics(env: &Environment) -> LasaResult<()> {
    let data = match read(&env.data_file_path) {
        Ok(data) => data.into_object().unwrap(),
        Err(err) => return Err(LasaError::DataStorage(err.to_string())),
    }
}
