use std::env;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;

use serde_json::Value;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::{Child, Command};
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

pub(crate) async fn analyze_urls(
    urls: Vec<String>,
    settings: &AppSettings,
    cancel_token: Signal<u64>,
    cancel_generation: u64,
) -> Result<Option<AnalysisResult>, AppError> {
    if urls.is_empty() {
        return Err(AppError::new(
            i18n::t("error_no_url_title"),
            i18n::t("error_no_url_message"),
            "",
        ));
    }

    if analysis_was_cancelled(cancel_token, cancel_generation) {
        return Ok(None);
    }

    ensure_ytdlp_dependencies().await?;

    let mut items = Vec::new();
    let mut warnings = Vec::new();

    for url in &urls {
        if analysis_was_cancelled(cancel_token, cancel_generation) {
            return Ok(None);
        }

        let args = build_analysis_args(url, settings);
        let mut command = yt_dlp_command(&args);
        let Some(output) =
            run_analysis_command(&mut command, cancel_token, cancel_generation).await?
        else {
            return Ok(None);
        };

        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        if !output.status.success() {
            return Err(AppError::new(
                i18n::t("analysis_failed"),
                first_nonempty_line(&stderr).unwrap_or_else(|| i18n::t("analysis_failed_message")),
                stderr,
            ));
        }

        warnings.extend(warning_lines(&stderr));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).map_err(|error| {
            AppError::new(
                i18n::t("analysis_invalid_json"),
                i18n::t("analysis_invalid_json_message"),
                format!("{error}\n\n{stdout}\n\n{stderr}"),
            )
        })?;

        let parsed = parse_analysis_json(&json, url);
        if parsed.is_empty() {
            warnings.push(i18n::t_with("no_entries_found", &[("url", url.clone())]));
        }
        items.extend(parsed);
    }

    if items.is_empty() {
        return Err(AppError::new(
            i18n::t("no_downloadable_media"),
            i18n::t("no_downloadable_media_message"),
            warnings.join("\n"),
        ));
    }

    Ok(Some(AnalysisResult {
        source_label: if items.len() > 1 {
            i18n::t_with("analyzed_item_count", &[("count", items.len().to_string())])
        } else {
            items[0].title.clone()
        },
        items,
        command: yt_dlp_command_display(&build_analysis_args(&urls[0], settings)),
        warnings,
    }))
}

pub(crate) async fn run_analysis_command(
    command: &mut Command,
    cancel_token: Signal<u64>,
    cancel_generation: u64,
) -> Result<Option<Output>, AppError> {
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            AppError::new(
                i18n::t("yt_dlp_not_available_title"),
                i18n::t("yt_dlp_not_available_message"),
                error.to_string(),
            )
        })?;

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let stdout_task = tokio::spawn(read_stream_to_end(stdout));
    let stderr_task = tokio::spawn(read_stream_to_end(stderr));

    loop {
        if analysis_was_cancelled(cancel_token, cancel_generation) {
            cancel_child(&mut child);
            stdout_task.abort();
            stderr_task.abort();
            return Ok(None);
        }

        match child.try_wait() {
            Ok(Some(status)) => {
                let stdout = stdout_task.await.unwrap_or_default();
                let stderr = stderr_task.await.unwrap_or_default();
                return Ok(Some(Output {
                    status,
                    stdout,
                    stderr,
                }));
            }
            Ok(None) => {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
            Err(error) => {
                cancel_child(&mut child);
                stdout_task.abort();
                stderr_task.abort();
                return Err(AppError::new(
                    i18n::t("analysis_failed"),
                    i18n::t("analysis_failed_message"),
                    error.to_string(),
                ));
            }
        }
    }
}

fn analysis_was_cancelled(cancel_token: Signal<u64>, cancel_generation: u64) -> bool {
    (cancel_token)() != cancel_generation
}

async fn read_stream_to_end<R>(reader: Option<R>) -> Vec<u8>
where
    R: AsyncRead + Unpin,
{
    let Some(mut reader) = reader else {
        return Vec::new();
    };

    let mut output = Vec::new();
    let _ = reader.read_to_end(&mut output).await;
    output
}

fn cancel_child(child: &mut Child) {
    let _ = child.start_kill();
}

pub(crate) fn yt_dlp_command(args: &[String]) -> Command {
    let paths = managed_binary_paths();
    let mut command = Command::new(&paths.ytdlp);
    command
        .args(yt_dlp_args(args, &paths))
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONUTF8", "1");
    hide_process_window(&mut command);
    command
}

pub(crate) fn build_analysis_args(url: &str, settings: &AppSettings) -> Vec<String> {
    let mut args = vec![
        "--no-update".to_string(),
        "--encoding".to_string(),
        "utf-8".to_string(),
        "--dump-single-json".to_string(),
        "--skip-download".to_string(),
    ];
    add_common_network_args(&mut args, settings);
    args.push(url.to_string());
    args
}

pub(crate) fn add_common_network_args(args: &mut Vec<String>, settings: &AppSettings) {
    if !settings.cookie_file.trim().is_empty() {
        args.push("--cookies".to_string());
        args.push(settings.cookie_file.clone());
    }
    if !settings.proxy.trim().is_empty() {
        args.push("--proxy".to_string());
        args.push(settings.proxy.clone());
    }
}

pub(crate) fn parse_analysis_json(json: &Value, fallback_url: &str) -> Vec<MediaItem> {
    if let Some(entries) = json.get("entries").and_then(Value::as_array) {
        entries
            .iter()
            .filter(|entry| !entry.is_null())
            .map(|entry| media_item_from_json(entry, fallback_url))
            .collect()
    } else {
        vec![media_item_from_json(json, fallback_url)]
    }
}

pub(crate) fn media_item_from_json(json: &Value, fallback_url: &str) -> MediaItem {
    let title = json
        .get("title")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| i18n::t("untitled_media"));
    let uploader = json
        .get("uploader")
        .or_else(|| json.get("channel"))
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .unwrap_or_else(|| i18n::t("unknown"));
    let url = json
        .get("webpage_url")
        .or_else(|| json.get("original_url"))
        .or_else(|| json.get("url"))
        .and_then(Value::as_str)
        .unwrap_or(fallback_url)
        .to_string();
    let duration = json
        .get("duration")
        .and_then(Value::as_f64)
        .map(format_duration)
        .unwrap_or_else(|| "-".to_string());
    let format_count = json
        .get("formats")
        .and_then(Value::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    let thumbnail = json
        .get("thumbnail")
        .and_then(Value::as_str)
        .map(ToString::to_string)
        .or_else(|| best_thumbnail(json))
        .unwrap_or_default();
    let estimated_size = estimate_size(json);

    MediaItem {
        title,
        uploader,
        url,
        duration,
        thumbnail,
        format_count,
        estimated_size,
        selected: true,
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

pub(crate) fn yt_dlp_args(args: &[String], paths: &BinaryPaths) -> Vec<String> {
    let mut full_args = Vec::with_capacity(args.len() + 2);
    full_args.push("--ffmpeg-location".to_string());
    full_args.push(path_to_arg(&paths.ffmpeg_dir));
    full_args.extend(args.iter().cloned());
    full_args
}

pub(crate) fn yt_dlp_command_display(args: &[String]) -> String {
    let paths = managed_binary_paths();
    let binary = path_to_arg(&paths.ytdlp);
    let full_args = yt_dlp_args(args, &paths);
    display_command(&binary, &full_args)
}

pub(crate) fn display_command(binary: &str, args: &[String]) -> String {
    std::iter::once(binary.to_string())
        .chain(args.iter().map(|arg| quote_arg(arg)))
        .collect::<Vec<_>>()
        .join(" ")
}

pub(crate) fn quote_arg(arg: &str) -> String {
    if arg
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || "-_./:=+,%[]".contains(ch))
    {
        arg.to_string()
    } else {
        format!("\"{}\"", arg.replace('"', "\\\""))
    }
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

fn first_nonempty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToString::to_string)
}

fn warning_lines(stderr: &str) -> Vec<String> {
    stderr
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

fn format_duration(seconds: f64) -> String {
    let total = seconds.round().max(0.0) as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

fn estimate_size(json: &Value) -> String {
    let bytes = json
        .get("filesize_approx")
        .or_else(|| json.get("filesize"))
        .and_then(Value::as_u64)
        .or_else(|| {
            json.get("formats")
                .and_then(Value::as_array)
                .and_then(|formats| {
                    formats
                        .iter()
                        .filter_map(|format| {
                            format
                                .get("filesize")
                                .or_else(|| format.get("filesize_approx"))
                                .and_then(Value::as_u64)
                        })
                        .max()
                })
        });

    match bytes {
        Some(bytes) if bytes > 1_000_000_000 => format!("{:.1} GB", bytes as f64 / 1_000_000_000.0),
        Some(bytes) if bytes > 1_000_000 => format!("{:.1} MB", bytes as f64 / 1_000_000.0),
        Some(bytes) if bytes > 0 => format!("{:.1} KB", bytes as f64 / 1_000.0),
        _ => i18n::t("unknown"),
    }
}

fn best_thumbnail(json: &Value) -> Option<String> {
    json.get("thumbnails")
        .and_then(Value::as_array)?
        .iter()
        .rev()
        .find_map(|thumbnail| {
            thumbnail
                .get("url")
                .and_then(Value::as_str)
                .map(ToString::to_string)
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

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

    #[test]
    fn parses_single_video_metadata() {
        let item = media_item_from_json(
            &json!({
                "title": "A video",
                "uploader": "A channel",
                "webpage_url": "https://example.com/watch",
                "duration": 125,
                "thumbnail": "https://example.com/thumb.jpg",
                "filesize_approx": 2_500_000,
                "formats": [{ "format_id": "18" }, { "format_id": "22" }]
            }),
            "https://fallback.test",
        );

        assert_eq!(item.title, "A video");
        assert_eq!(item.uploader, "A channel");
        assert_eq!(item.url, "https://example.com/watch");
        assert_eq!(item.duration, "2:05");
        assert_eq!(item.thumbnail, "https://example.com/thumb.jpg");
        assert_eq!(item.format_count, 2);
        assert_eq!(item.estimated_size, "2.5 MB");
        assert!(item.selected);
    }

    #[test]
    fn parses_playlist_entries_and_skips_null_items() {
        let items = parse_analysis_json(
            &json!({
                "entries": [
                    null,
                    { "title": "One", "channel": "Uploader", "duration": 3661, "url": "https://example.com/one" },
                    { "title": "Two", "formats": [{ "filesize": 1_500_000_000u64 }] }
                ]
            }),
            "https://fallback.test",
        );

        assert_eq!(items.len(), 2);
        assert_eq!(items[0].title, "One");
        assert_eq!(items[0].uploader, "Uploader");
        assert_eq!(items[0].duration, "1:01:01");
        assert_eq!(items[1].url, "https://fallback.test");
        assert_eq!(items[1].estimated_size, "1.5 GB");
    }

    #[test]
    fn builds_analysis_args_with_common_network_options() {
        let mut settings = AppSettings::default();
        settings.cookie_file = "cookies.txt".to_string();
        settings.proxy = "socks5://localhost:9000".to_string();

        assert_eq!(
            build_analysis_args("https://example.com/video", &settings),
            vec![
                "--no-update",
                "--encoding",
                "utf-8",
                "--dump-single-json",
                "--skip-download",
                "--cookies",
                "cookies.txt",
                "--proxy",
                "socks5://localhost:9000",
                "https://example.com/video",
            ]
        );
    }
}
