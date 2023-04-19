use std::fs;

use anyhow::{Context, Result};
use log::{debug, info};

use super::paths;

const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

pub async fn process_cache_cleanup() -> Result<()> {
    let Some(app_version) = get_app_version().await? else {
        debug!("No app version file found, cleaning up cache and creating app version file");
        clean_up_cache().await?;
        store_app_version().await?;
        return Ok(());
    };

    if app_version != APP_VERSION {
        info!("Different set up tool version detected, old: {app_version}, current: {APP_VERSION}. cleaning up cache");
        clean_up_cache().await?;
        store_app_version().await?;
    }

    Ok(())
}

async fn store_app_version() -> Result<()> {
    let app_version = env!("CARGO_PKG_VERSION");
    let app_version_path =
        paths::app_version_file().context("Could not get app version file path")?;
    fs::write(app_version_path, app_version).context("Could not write app version file")?;
    Ok(())
}

// either has a version or never had that file
async fn get_app_version() -> Result<Option<String>> {
    let app_version_path =
        paths::app_version_file().context("Could not get app version file path")?;

    if !app_version_path.exists() {
        return Ok(None);
    }

    let app_version = fs::read_to_string(app_version_path).context("Could not read app version")?;
    Ok(Some(app_version))
}

async fn clean_up_cache() -> Result<()> {
    let data_storage_dir =
        paths::data_storage_dir().context("Could not get data storage directory")?;

    fs::remove_dir_all(&data_storage_dir).context("Could not remove data storage directory")?;
    fs::create_dir_all(data_storage_dir).context("Could not create data storage directory")?;

    Ok(())
}
