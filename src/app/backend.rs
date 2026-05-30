use std::env;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;

use futures_util::{stream, StreamExt as _};
use serde_json::Value;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
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

pub(crate) async fn run_queue(
    jobs: Signal<Vec<DownloadJob>>,
    settings: AppSettings,
    failed_only: bool,
    mut last_error: Signal<Option<AppError>>,
) {
    let pending_ids = queue_target_ids(jobs, failed_only);
    if pending_ids.is_empty() {
        return;
    }

    if let Err(error) = ensure_ytdlp_dependencies().await {
        for id in pending_ids {
            set_job_failed(jobs, id, error.clone());
        }
        last_error.set(Some(error));
        return;
    }

    let limit = settings.parallel_jobs.max(1) as usize;
    stream::iter(pending_ids)
        .for_each_concurrent(limit, move |id| {
            let jobs = jobs;
            let settings = settings.clone();
            let mut last_error = last_error;

            async move {
                if let Err(error) = run_download_job(jobs, id, &settings).await {
                    last_error.set(Some(error));
                }
            }
        })
        .await;
}

fn queue_target_ids(mut jobs: Signal<Vec<DownloadJob>>, failed_only: bool) -> Vec<u64> {
    let mut ids = Vec::new();

    jobs.with_mut(|items| {
        for job in items {
            let should_run = if failed_only {
                job.status == JobStatus::Failed
            } else {
                job.status == JobStatus::Queued
            };

            if !should_run {
                continue;
            }

            if failed_only {
                job.status = JobStatus::Queued;
                job.progress = 0.0;
                job.speed = "-".to_string();
                job.eta = "-".to_string();
                job.step = i18n::t("job_step_queued_retry");
                job.error = None;
                job.log.push(i18n::t("job_step_queued_retry"));
            }

            ids.push(job.id);
        }
    });

    ids
}

pub(crate) async fn run_download_job(
    jobs: Signal<Vec<DownloadJob>>,
    id: u64,
    settings: &AppSettings,
) -> Result<(), AppError> {
    let job_snapshot = jobs
        .read()
        .iter()
        .find(|job| job.id == id)
        .cloned()
        .ok_or_else(|| AppError::new(i18n::t("queue_error"), i18n::t("queued_job_missing"), ""))?;

    if let Err(error) = std::fs::create_dir_all(&job_snapshot.output_folder) {
        let app_error = AppError::new(
            i18n::t("output_folder_not_writable"),
            i18n::t_with(
                "could_not_create_folder",
                &[("path", job_snapshot.output_folder.clone())],
            ),
            error.to_string(),
        );
        set_job_failed(jobs, id, app_error.clone());
        return Err(app_error);
    }

    let args = build_download_args(&job_snapshot, settings);

    update_job(jobs, id, |job| {
        job.status = JobStatus::Running;
        job.progress = 0.0;
        job.speed = "-".to_string();
        job.eta = "-".to_string();
        job.step = i18n::t("job_step_starting");
        job.command_display = yt_dlp_command_display(&args);
        job.log.push(job.command_display.clone());
    });

    let mut command = yt_dlp_command(&args);
    let mut child = match command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(error) => {
            let app_error = AppError::new(
                i18n::t("yt_dlp_could_not_start"),
                i18n::t("yt_dlp_could_not_start_message"),
                error.to_string(),
            );
            set_job_failed(jobs, id, app_error.clone());
            return Err(app_error);
        }
    };

    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    tokio::join!(
        read_process_stream(jobs, id, stdout),
        read_process_stream(jobs, id, stderr)
    );

    let status = match child.wait().await {
        Ok(status) => status,
        Err(error) => {
            let app_error = AppError::new(
                i18n::t("yt_dlp_process_failed"),
                i18n::t("yt_dlp_process_failed_message"),
                error.to_string(),
            );
            set_job_failed(jobs, id, app_error.clone());
            return Err(app_error);
        }
    };

    if status.success() {
        update_job(jobs, id, |job| {
            job.status = JobStatus::Completed;
            job.progress = 100.0;
            job.step = i18n::t("job_step_completed");
            job.speed = "-".to_string();
            job.eta = "-".to_string();
        });
        Ok(())
    } else {
        let debug = job_log(jobs, id);
        let mut app_error = AppError::new(
            i18n::t("download_failed"),
            first_error_line(&debug)
                .or_else(|| first_nonempty_line(&debug))
                .unwrap_or_else(|| i18n::t("download_failed_message")),
            debug,
        );
        app_error.suggestion = friendly_suggestion(&app_error.debug);
        set_job_failed(jobs, id, app_error.clone());
        Err(app_error)
    }
}

pub(crate) async fn read_process_stream<R>(
    jobs: Signal<Vec<DownloadJob>>,
    id: u64,
    reader: Option<R>,
) where
    R: AsyncRead + Unpin,
{
    let Some(reader) = reader else {
        return;
    };

    let mut reader = BufReader::new(reader);
    let mut buffer = Vec::with_capacity(1024);

    loop {
        buffer.clear();
        match reader.read_until(b'\n', &mut buffer).await {
            Ok(0) => break,
            Ok(_) => {
                while matches!(buffer.last(), Some(b'\n' | b'\r')) {
                    buffer.pop();
                }
                let line = String::from_utf8_lossy(&buffer);
                apply_process_line(jobs, id, &line);
            }
            Err(error) => {
                update_job(jobs, id, |job| {
                    job.log.push(i18n::t_with(
                        "could_not_read_ytdlp_output",
                        &[("error", error.to_string())],
                    ));
                });
                break;
            }
        }
    }
}

pub(crate) fn apply_process_line(jobs: Signal<Vec<DownloadJob>>, id: u64, line: &str) {
    let clean = line.trim().to_string();
    if clean.is_empty() {
        return;
    }

    update_job(jobs, id, |job| {
        if job.log.len() > 400 {
            job.log.remove(0);
        }
        job.log.push(clean.clone());

        if let Some(percent) = parse_percent(&clean) {
            job.progress = percent.clamp(0.0, 100.0);
            job.step = i18n::t("job_step_downloading");
        }
        if clean.contains("[Merger]") || clean.to_ascii_lowercase().contains("merging") {
            job.step = i18n::t("job_step_merging");
            job.progress = job.progress.max(95.0);
        }
        if clean.contains("[ExtractAudio]") {
            job.step = i18n::t("job_step_extracting_audio");
            job.progress = job.progress.max(95.0);
        }
        if clean.to_ascii_lowercase().contains("embedding") {
            job.step = i18n::t("job_step_embedding_metadata");
            job.progress = job.progress.max(96.0);
        }
        if let Some(speed) = parse_between(&clean, " at ", " ETA ") {
            job.speed = speed;
        }
        if let Some(eta) = parse_after(&clean, " ETA ") {
            job.eta = eta;
        }
        if let Some(destination) = process_destination(&clean) {
            job.output_hint = destination;
        }
    });
}

pub(crate) fn update_job(
    mut jobs: Signal<Vec<DownloadJob>>,
    id: u64,
    mut update: impl FnMut(&mut DownloadJob),
) {
    jobs.with_mut(|items| {
        if let Some(job) = items.iter_mut().find(|job| job.id == id) {
            update(job);
        }
    });
}

pub(crate) fn set_job_failed(jobs: Signal<Vec<DownloadJob>>, id: u64, error: AppError) {
    update_job(jobs, id, |job| {
        job.status = JobStatus::Failed;
        job.step = error.title.clone();
        job.speed = "-".to_string();
        job.eta = "-".to_string();
        job.error = Some(error.clone());
        job.log.push(error.message.clone());
    });
}

fn job_log(jobs: Signal<Vec<DownloadJob>>, id: u64) -> String {
    jobs.read()
        .iter()
        .find(|job| job.id == id)
        .map(|job| job.log.join("\n"))
        .unwrap_or_default()
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

fn first_error_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_ascii_lowercase();
            lower.starts_with("error:")
                || lower.contains("unable to")
                || lower.contains("failed")
                || lower.contains("errno")
        })
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

fn parse_percent(line: &str) -> Option<f32> {
    let percent_index = line.find('%')?;
    let before = &line[..percent_index];
    let number: String = before
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    number.parse::<f32>().ok()
}

fn parse_between(line: &str, start: &str, end: &str) -> Option<String> {
    let start_index = line.find(start)? + start.len();
    let rest = &line[start_index..];
    let end_index = rest.find(end)?;
    Some(rest[..end_index].trim().to_string())
}

fn parse_after(line: &str, marker: &str) -> Option<String> {
    let index = line.find(marker)? + marker.len();
    Some(line[index..].trim().to_string())
}

fn process_destination(line: &str) -> Option<String> {
    let destination = line
        .strip_prefix("[download] Destination:")
        .or_else(|| line.strip_prefix("[ExtractAudio] Destination:"))
        .or_else(|| line.strip_prefix("[Merger] Merging formats into"))?
        .trim()
        .trim_matches('"');

    if destination.is_empty() {
        None
    } else {
        Some(destination.to_string())
    }
}

fn friendly_suggestion(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("ffmpeg") {
        i18n::t("suggestion_ffmpeg")
    } else if lower.contains("yt-dlp") {
        i18n::t("suggestion_ytdlp")
    } else if lower.contains("cookies") || lower.contains("login") || lower.contains("private") {
        i18n::t("suggestion_cookies")
    } else if lower.contains("format") {
        i18n::t("suggestion_format")
    } else if lower.contains("network") || lower.contains("timed out") {
        i18n::t("suggestion_network")
    } else {
        i18n::t("suggestion_default")
    }
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

    #[test]
    fn parses_download_progress_lines() {
        let line = "[download]  42.7% of 10.00MiB at 1.25MiB/s ETA 00:08";

        assert_eq!(parse_percent(line), Some(42.7));
        assert_eq!(
            parse_between(line, " at ", " ETA "),
            Some("1.25MiB/s".to_string())
        );
        assert_eq!(parse_after(line, " ETA "), Some("00:08".to_string()));
    }

    #[test]
    fn parses_download_destinations() {
        assert_eq!(
            process_destination("[download] Destination: C:\\Downloads\\video.mp4"),
            Some("C:\\Downloads\\video.mp4".to_string())
        );
        assert_eq!(
            process_destination("[Merger] Merging formats into \"C:\\Downloads\\merged.mkv\""),
            Some("C:\\Downloads\\merged.mkv".to_string())
        );
    }

    #[test]
    fn finds_download_error_lines() {
        assert_eq!(
            first_error_line("noise\nERROR: requested format is not available\nmore"),
            Some("ERROR: requested format is not available".to_string())
        );
    }
}
