use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{bail, Context, Result};
use log::*;

use crate::utils::{
    self,
    download::{self, UNITAS_BEPINEX_DIR},
};

use super::{
    cli::{DownloadVersion, GameDirSelection},
    game_dir::dir_info::{DirInfo, GamePlatform},
    local_versions::LocalVersions,
    paths,
};

pub async fn install(
    game_dir: GameDirSelection,
    unitas_version: DownloadVersion,
    bepinex_version: DownloadVersion,
    offline: bool,
) -> Result<()> {
    let game_dir = PathBuf::try_from(game_dir)?;
    let dir_info = DirInfo::from_dir(&game_dir)?;

    if !dir_info.is_unity_dir {
        bail!("Game directory is not a Unity game directory");
    }

    if matches!(&dir_info.game_platform, GamePlatform::Unknown) {
        bail!("Game platform is unknown");
    }

    {
        let bepinex_version = bepinex_version.clone();
        let unitas_version = unitas_version.clone();

        let download_tasks = [
            tokio::spawn(async move { dl_bepinex_if_missing(bepinex_version, offline).await }),
            tokio::spawn(async move { dl_unitas_if_missing(unitas_version, offline).await }),
        ];

        for task in download_tasks {
            task.await.unwrap()?;
        }
    }

    info!("Installing BepInEx");
    install_bepinex(&game_dir, bepinex_version, &dir_info).await?;
    info!("Installed BepInEx successfully");

    info!("Installing UniTAS");
    install_unitas(&game_dir, unitas_version).await?;
    info!("Installed UniTAS successfully");

    Ok(())
}

async fn dl_bepinex_if_missing(bepinex_version: DownloadVersion, offline: bool) -> Result<()> {
    let installed_bepinex = LocalVersions::from_dir(&paths::local_bepinex_dir()?)?;

    if !installed_bepinex.versions.contains(&bepinex_version) {
        // download if not offline
        if offline {
            bail!("BepInEx version not found and offline mode is enabled");
        }

        download::download_bepinex(&bepinex_version).await?;
    }

    Ok(())
}

async fn dl_unitas_if_missing(unitas_version: DownloadVersion, offline: bool) -> Result<()> {
    let installed_unitas = LocalVersions::from_dir(&paths::local_unitas_dir()?)?;

    if !installed_unitas.versions.contains(&unitas_version) {
        // download if not offline
        if offline {
            bail!("UniTAS version not found and offline mode is enabled");
        }

        download::download_unitas(&unitas_version).await?;
    }

    Ok(())
}

async fn install_bepinex(
    game_dir: &Path,
    bepinex_version: DownloadVersion,
    dir_info: &DirInfo,
) -> Result<()> {
    // overwrite install without overwriting the config and other important files
    let bepinex_dir = paths::local_bepinex_dir()?;
    let bepinex_dir = match bepinex_version {
        DownloadVersion::Stable => bepinex_dir.join(paths::STABLE_DIR_NAME),
        DownloadVersion::Tag(tag) => bepinex_dir.join(paths::TAG_DIR_NAME).join(tag),
        DownloadVersion::Branch(branch) => bepinex_dir.join(paths::BRANCH_DIR_NAME).join(branch),
    };
    let bepinex_dir = {
        // update this when we support IL2Cpp
        let files = fs::read_dir(bepinex_dir)?.collect::<Result<Vec<_>, _>>()?;

        let platform = match &dir_info.game_platform {
            GamePlatform::Unix => "unix",
            GamePlatform::Windowsx64 => "x64",
            GamePlatform::Windowsx86 => "x86",
            GamePlatform::Unknown => unreachable!(),
        };

        files
            .iter()
            .find_map(|entry| {
                let path = entry.path();
                let file_name = path.file_name()?.to_string_lossy();

                let exclusion = ["IL2CPP", "NetLauncher", "Framework"];

                if path.is_file()
                    || exclusion.contains(&file_name.as_ref())
                    || !file_name.contains(platform)
                {
                    return None;
                }

                Some(path)
            })
            .context("Could not find a BepInEx dir with the correct platform")
    }?;

    // path to not overwrite if it exists
    let dont_overwrite_paths = [
        Path::new("run_bepinex.sh").to_string_lossy().to_string(),
        Path::new("BepInEx")
            .join("config")
            .to_string_lossy()
            .to_string(),
    ];

    let mut tasks = Vec::new();

    // remove cache dir
    {
        let game_dir = game_dir.to_owned();
        tasks.push(tokio::spawn(async move {
            let cache_dir = game_dir.join("BepInEx").join("cache");
            if cache_dir.exists() {
                info!("Removing BepInEx cache dir: {}", cache_dir.display());
                fs::remove_dir_all(cache_dir).context("Could not remove BepInEx cache dir")?;
            }
            Ok(())
        }));
    }

    let bepinex_files = fs::read_dir(&bepinex_dir)?;
    for bepinex_path in bepinex_files {
        let bepinex_path = bepinex_path.with_context(|| {
            format!("Could not read BepInEx file from {}", bepinex_dir.display())
        })?;

        let bepinex_path = bepinex_path.path();
        debug!("Processing BepInEx file: {}", bepinex_path.display());
        let file_name = if let Some(file_name) = bepinex_path.file_name() {
            file_name
        } else {
            debug!("Skipping BepInEx file: {}", bepinex_path.display());
            continue;
        };

        let dest_path = game_dir.join(file_name);

        // ignore check
        if dest_path.exists()
            && dont_overwrite_paths.contains(&file_name.to_string_lossy().to_string())
        {
            debug!("Skipping BepInEx file: {}", bepinex_path.display());
            continue;
        }

        tasks.push(tokio::spawn(async move {
            debug!(
                "Copying {} to {}",
                bepinex_path.display(),
                dest_path.display()
            );
            if bepinex_path.is_dir() {
                debug!("{} is a dir", bepinex_path.display());
                utils::fs::copy_dir_all(&bepinex_path, &dest_path, true)?;
            } else {
                debug!("{} is a file", bepinex_path.display());
                fs::copy(&bepinex_path, &dest_path).with_context(|| {
                    format!(
                        "Could not copy BepInEx file from {} to {}",
                        bepinex_path.display(),
                        dest_path.display()
                    )
                })?;
            }

            Ok::<_, anyhow::Error>(())
        }));
    }

    for task in tasks {
        task.await.unwrap()?;
    }

    Ok(())
}

async fn install_unitas(game_dir: &Path, unitas_version: DownloadVersion) -> Result<()> {
    // overwrite install without overwriting the config and other important files
    let unitas_dir = paths::local_unitas_dir()?;

    let unitas_dir = match unitas_version {
        DownloadVersion::Stable => {
            let unitas_dir = unitas_dir.join(paths::STABLE_DIR_NAME);

            fs::read_dir(&unitas_dir)?
                .collect::<Result<Vec<_>, _>>()
                .with_context(|| {
                    format!(
                        "Could not read dir: {}, can't get BepInEx dirs",
                        unitas_dir.display()
                    )
                })?
                .iter()
                .find_map(|entry| {
                    let path = entry.path();
                    let Some(file_name) = path.file_name() else {
                        return None;
                    };
                    let file_name = file_name.to_string_lossy().to_string();
                    if file_name.starts_with(UNITAS_BEPINEX_DIR) {
                        Some(path)
                    } else {
                        None
                    }
                })
                .with_context(|| {
                    format!(
                        "Could not find a UniTAS BepInEx dir in {}",
                        unitas_dir.display()
                    )
                })?
        }
        DownloadVersion::Tag(tag) => unitas_dir.join(paths::TAG_DIR_NAME).join(tag),
        DownloadVersion::Branch(branch) => unitas_dir
            .join(paths::BRANCH_DIR_NAME)
            .join(branch)
            .join(format!(
                "{}-{}",
                download::UNITAS_UNIX_BUILD_RELEASE,
                download::UNITAS_BUILD_RELEASE
            )),
    };

    let dest_dir = game_dir.join("BepInEx");

    let source_dest_dirs = [
        (
            unitas_dir.join(paths::unitas_plugins_dir()),
            (dest_dir.join("plugins").join("UniTAS")),
        ),
        (
            unitas_dir.join(paths::unitas_patchers_dir()),
            (dest_dir.join("patchers").join("UniTAS")),
        ),
    ];

    for (source_dir, dest_dir) in source_dest_dirs {
        debug!(
            "Copying contents of {} to {}",
            source_dir.display(),
            dest_dir.display()
        );

        utils::fs::copy_dir_all(&source_dir, &dest_dir, true)?;
    }

    Ok(())
}
