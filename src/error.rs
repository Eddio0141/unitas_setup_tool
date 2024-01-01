//! Main errors

use crate::utils::*;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Path(#[from] paths::error::Error),
    #[error(transparent)]
    History(#[from] history::error::Error),
    #[error(transparent)]
    GameDir(#[from] game_dir::error::Error),
    #[error("Failed to download UniTAS: {0}")]
    DownloadUniTAS(#[from] download::error::Error),
    #[error(transparent)]
    LocalVersions(#[from] local_versions::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
