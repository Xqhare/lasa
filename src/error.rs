pub type LasaResult<T> = std::result::Result<T, LasaError>;

#[allow(dead_code)]
#[derive(Debug)]
pub enum LasaError {
    EnvironmentError(String),
    SettingProcessPriority(String),
    Generic(String),
    CommandExecution(String),
    Parsing(String),
    DataStorage(String),
    JournalProbeFailed,
    NoData(String),
}

impl std::fmt::Display for LasaError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LasaError::EnvironmentError(err) => write!(f, "Environment error: {}", err),
            LasaError::SettingProcessPriority(err) => write!(f, "Process priority error: {}", err),
            LasaError::Generic(err) => write!(f, "Error: {}", err),
            LasaError::CommandExecution(err) => write!(f, "Command execution error: {}", err),
            LasaError::Parsing(err) => write!(f, "Parsing error: {}", err),
            LasaError::DataStorage(err) => write!(f, "Data storage error: {}", err),
            LasaError::JournalProbeFailed => write!(f, "Journal probe failed"),
            LasaError::NoData(err) => write!(f, "No data: {}", err),
        }
    }
}

impl std::error::Error for LasaError {}
