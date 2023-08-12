use std::fmt::{self, Write};
use std::path::PathBuf;

#[derive(thiserror::Error)]
pub enum Error {
    #[error("invalid regex {0}")]
    Regex(#[from] regex::Error),

    #[error(transparent)]
    File(#[from] std::io::Error),

    #[error("failed to move file: {0}")]
    TempfilePersist(#[from] tempfile::PersistError),

    #[error("file doesn't have parent path: {0}")]
    InvalidPath(PathBuf),

    #[error("failed processing files:\n{0}")]
    FailedProcessing(FailedJobs),

    #[error("failed walking directories:\n{0}")]
    Walkdir(#[from] walkdir::Error),
}

pub struct FailedJobs(pub Vec<(Option<PathBuf>, Error)>);

impl fmt::Display for FailedJobs {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("\tFailedJobs(\n")?;
        for (path, err) in &self.0 {
            f.write_str(&format!("\t{:?}: {}\n", path, err))?;
        }
        f.write_char(')')
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
