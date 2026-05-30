use std::env;
use std::path::{Path, PathBuf};

use tokio::process::Command;
use yt_dlp::client::deps::Libraries as YtDlpLibraries;

use super::*;

const MANAGED_BIN_DIR: &str = "yaydlp/bin";
const YTDLP_BIN: &str = "yt-dlp";
const FFMPEG_BIN: &str = "ffmpeg";
#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct BinaryPaths {
    pub(crate) ytdlp: PathBuf,
    pub(crate) ffmpeg_dir: PathBuf,
    pub(crate) ffmpeg: PathBuf,
}

pub(crate) async fn check_dependencies() -> DependencyReport {
    let paths = managed_binary_paths();
    let install_error = ensure_ytdlp_dependencies().await.err();
    let mut report = DependencyReport {
        ytdlp: check_binary("yt-dlp", &paths.ytdlp, &["--version"]).await,
        ffmpeg: check_binary("FFmpeg", &paths.ffmpeg, &["-version"]).await,
    };

    if let Some(error) = install_error {
        if !report.ytdlp.installed {
            report.ytdlp.detail = dependency_install_failed_detail(&paths.ytdlp, &error);
        }
        if !report.ffmpeg.installed {
            report.ffmpeg.detail = dependency_install_failed_detail(&paths.ffmpeg, &error);
        }
    }

    report
}

pub(crate) async fn ensure_ytdlp_dependencies() -> Result<BinaryPaths, AppError> {
    let paths = managed_binary_paths();
    let libraries = YtDlpLibraries::new(paths.ytdlp.clone(), paths.ffmpeg.clone());

    libraries.install_dependencies().await.map_err(|error| {
        AppError::new(
            i18n::t("dependency_install_failed"),
            i18n::t("dependency_install_failed_message"),
            error.to_string(),
        )
    })?;

    Ok(managed_binary_paths())
}

pub(crate) async fn check_binary(name: &str, binary: &Path, args: &[&str]) -> DependencyItem {
    let mut command = Command::new(binary);
    command.args(args);
    hide_process_window(&mut command);

    match command.output().await {
        Ok(output) if output.status.success() => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let stderr = String::from_utf8_lossy(&output.stderr);
            let raw = if stdout.trim().is_empty() {
                stderr.trim()
            } else {
                stdout.trim()
            };
            let version = raw
                .lines()
                .next()
                .map(str::trim)
                .filter(|line| !line.is_empty())
                .map(ToString::to_string)
                .unwrap_or_else(|| i18n::t("installed"));

            DependencyItem {
                name: name.to_string(),
                installed: true,
                version: short_version(&version),
                detail: i18n::t_with(
                    "dependency_detail_from",
                    &[
                        ("version", version),
                        ("path", bundle_relative_display(binary)),
                    ],
                ),
            }
        }
        Ok(output) => DependencyItem {
            name: name.to_string(),
            installed: false,
            version: i18n::t("missing"),
            detail: missing_binary_detail(binary, &String::from_utf8_lossy(&output.stderr)),
        },
        Err(error) => DependencyItem {
            name: name.to_string(),
            installed: false,
            version: i18n::t("missing"),
            detail: missing_binary_detail(binary, &error.to_string()),
        },
    }
}

pub(crate) fn hide_process_window(command: &mut Command) {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
    }
}

pub(crate) fn managed_binary_paths() -> BinaryPaths {
    let ffmpeg_dir = managed_bin_dir();
    BinaryPaths {
        ytdlp: ffmpeg_dir.join(executable_name(YTDLP_BIN)),
        ffmpeg: ffmpeg_dir.join(executable_name(FFMPEG_BIN)),
        ffmpeg_dir,
    }
}

pub(crate) fn managed_bin_dir() -> PathBuf {
    if let Some(root) = env::var_os("LOCALAPPDATA").or_else(|| env::var_os("APPDATA")) {
        return PathBuf::from(root).join(Path::new(MANAGED_BIN_DIR));
    }

    if let Some(profile) = env::var_os("USERPROFILE") {
        return PathBuf::from(profile)
            .join("AppData")
            .join("Local")
            .join(Path::new(MANAGED_BIN_DIR));
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join(Path::new(MANAGED_BIN_DIR));
    }

    env::current_dir()
        .unwrap_or_else(|_| PathBuf::from("."))
        .join(".yaydlp")
        .join("bin")
}

pub(crate) fn executable_name(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

pub(crate) fn path_to_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

fn bundle_relative_display(path: &Path) -> String {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(manifest_dir)
        .map(path_to_arg)
        .unwrap_or_else(|_| path_to_arg(path))
}

fn missing_binary_detail(path: &Path, detail: &str) -> String {
    let location = bundle_relative_display(path);
    let detail = detail.trim();
    if detail.is_empty() {
        i18n::t_with("missing_at", &[("path", location)])
    } else {
        i18n::t_with(
            "missing_at_detail",
            &[("path", location), ("detail", detail.to_string())],
        )
    }
}

fn dependency_install_failed_detail(path: &Path, error: &AppError) -> String {
    i18n::t_with(
        "dependency_install_failed_detail",
        &[("path", path_to_arg(path)), ("error", error.debug.clone())],
    )
}

fn short_version(version: &str) -> String {
    version
        .split_whitespace()
        .next()
        .map(ToString::to_string)
        .unwrap_or_else(|| i18n::t("installed"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_version_uses_first_token() {
        assert_eq!(short_version("ffmpeg version 7.1"), "ffmpeg");
        assert_eq!(short_version("2026.05.22"), "2026.05.22");
    }

    #[test]
    fn executable_name_matches_platform() {
        let expected = if cfg!(windows) {
            "yt-dlp.exe"
        } else {
            "yt-dlp"
        };

        assert_eq!(executable_name("yt-dlp"), expected);
    }
}
