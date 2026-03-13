use nabu::{Object, XffValue};

use crate::{
    env::Environment,
    error::{LasaError, LasaResult},
};

pub fn output_data(stats_ptr: &Object, env: &Environment) -> LasaResult<()> {
    let xff = XffValue::from(stats_ptr.clone());
    let xff_write = nabu::serde::write(&env.output_file_path, xff.clone());
    let json_write = mawu::write_pretty(&env.human_readable_output_path, xff, 4);
    if let Err(e) = xff_write {
        return Err(LasaError::DataStorage(e.to_string()));
    }
    if let Err(e) = json_write {
        return Err(LasaError::DataStorage(e.to_string()));
    }
    Ok(())
}
