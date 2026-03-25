use std::path::PathBuf;

use thiserror::Error;

use crate::target::Target;

#[derive(Debug, Error)]
pub enum Error {
    #[error("theme `{0}` is not configured")]
    UnknownTheme(String),
    #[error("no theme set. Run `hue set <theme>` first")]
    MissingCurrentTheme,
    #[error("theme `{0}` does not map to any supported targets")]
    NoSupportedTargets(String),
    #[error("missing target config for {app} at {path}")]
    MissingTarget { app: Target, path: PathBuf },
    #[error("unsupported {app} config layout: {reason}")]
    UnsupportedLayout { app: Target, reason: String },
}

pub type Result<T> = anyhow::Result<T>;
