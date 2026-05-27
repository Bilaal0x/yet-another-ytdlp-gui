use dioxus::prelude::*;
use futures_util::{future, stream, StreamExt};
use serde_json::Value;
use std::env;
use std::path::{Path, PathBuf};
use std::process::{Output, Stdio};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncReadExt, BufReader};
use tokio::process::{Child, Command};
use yt_dlp::client::deps::Libraries as YtDlpLibraries;

use super::*;

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

    let output = command.output().await;
    match output {
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

pub(crate) async fn analyze_urls(
    urls: Vec<String>,
    settings: &AppSettings,
    cancel_token: Signal<u64>,
    cancel_generation: u64,
) -> Result<Option<AnalysisResult>, AppError> {
    ensure_ytdlp_dependencies().await?;

    let mut items = Vec::new();
    let mut warnings = Vec::new();

    for url in &urls {
        let args = build_analysis_args(url, settings);
        let mut command = yt_dlp_command(&args);
        let Some(output) =
            run_analysis_command(&mut command, cancel_token, cancel_generation).await?
        else {
            return Ok(None);
        };

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();
            return Err(AppError::new(
                i18n::t("analysis_failed"),
                first_nonempty_line(&stderr).unwrap_or_else(|| i18n::t("analysis_failed_message")),
                stderr,
            ));
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let json: Value = serde_json::from_str(&stdout).map_err(|error| {
            AppError::new(
                i18n::t("analysis_invalid_json"),
                i18n::t("analysis_invalid_json_message"),
                format!("{error}\n\n{stdout}"),
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
    _failed_only: bool,
    mut last_error: Signal<Option<AppError>>,
) {
    let pending_ids: Vec<u64> = {
        jobs.read()
            .iter()
            .filter(|job| job.status == JobStatus::Queued)
            .map(|job| job.id)
            .collect()
    };

    if !pending_ids.is_empty() {
        if let Err(error) = ensure_ytdlp_dependencies().await {
            for id in pending_ids {
                set_job_failed(jobs, id, error.clone());
            }
            last_error.set(Some(error));
            return;
        }
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

    if let Err(error) = ensure_ytdlp_dependencies().await {
        set_job_failed(jobs, id, error.clone());
        return Err(error);
    }

    let preset = Preset {
        name: job_snapshot.format_label.clone(),
        kind: job_snapshot.download_type,
        format_label: job_snapshot.format_label.clone(),
        format_rule: format_rule(
            &job_snapshot.format_label,
            &job_snapshot.container,
            "Best",
            "1080p",
        ),
        audio_format: job_snapshot.audio_format.clone(),
        audio_quality: audio_quality_arg("320 kbps").to_string(),
        container: job_snapshot.container.clone(),
        output_template: job_snapshot.output_template.clone(),
        extra_flags: String::new(),
    };
    let args = build_download_args(&job_snapshot, settings, &preset);

    update_job(jobs, id, |job| {
        job.status = JobStatus::Running;
        job.step = "Starting yt-dlp".to_string();
        job.command_display = yt_dlp_command_display(&args);
        job.log.push(job.command_display.clone());
    });

    let mut command = yt_dlp_command(&args);
    let mut child = command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| {
            AppError::new(
                i18n::t("yt_dlp_could_not_start"),
                i18n::t("yt_dlp_could_not_start_message"),
                error.to_string(),
            )
        })?;

    future::join(
        read_process_stream(jobs, id, child.stdout.take()),
        read_process_stream(jobs, id, child.stderr.take()),
    )
    .await;

    let status = child.wait().await.map_err(|error| {
        AppError::new(
            i18n::t("yt_dlp_process_failed"),
            i18n::t("yt_dlp_process_failed_message"),
            error.to_string(),
        )
    })?;

    if status.success() {
        update_job(jobs, id, |job| {
            job.status = JobStatus::Completed;
            job.progress = 100.0;
            job.step = "Completed".to_string();
            job.speed = "-".to_string();
            job.eta = "-".to_string();
        });
        Ok(())
    } else {
        let debug = jobs
            .read()
            .iter()
            .find(|job| job.id == id)
            .map(|job| job.log.join("\n"))
            .unwrap_or_default();
        let app_error = AppError::new(
            i18n::t("download_failed"),
            first_error_line(&debug)
                .or_else(|| first_nonempty_line(&debug))
                .unwrap_or_else(|| i18n::t("download_failed_message")),
            debug,
        );
        set_job_failed(jobs, id, app_error.clone());
        Err(app_error)
    }
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

pub(crate) fn hide_process_window(command: &mut Command) {
    #[cfg(windows)]
    {
        command.creation_flags(CREATE_NO_WINDOW);
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
            job.progress = percent;
            job.step = "Downloading".to_string();
        }
        if clean.contains("[Merger]") || clean.to_ascii_lowercase().contains("merging") {
            job.step = "Merging".to_string();
            job.progress = job.progress.max(95.0);
        }
        if clean.contains("[ExtractAudio]") {
            job.step = "Extracting audio".to_string();
            job.progress = job.progress.max(95.0);
        }
        if clean.to_ascii_lowercase().contains("embedding") {
            job.step = "Embedding metadata".to_string();
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
        job.error = Some(error.clone());
        job.log.push(error.message.clone());
    });
}

pub(crate) fn build_analysis_args(url: &str, settings: &AppSettings) -> Vec<String> {
    let mut args = vec![
        "--no-update".to_string(),
        "--encoding".to_string(),
        "utf-8".to_string(),
        "--dump-single-json".to_string(),
        "--skip-download".to_string(),
        "--no-warnings".to_string(),
    ];
    add_common_network_args(&mut args, settings);
    args.push(url.to_string());
    args
}

pub(crate) fn build_download_args(
    job: &DownloadJob,
    settings: &AppSettings,
    preset: &Preset,
) -> Vec<String> {
    let mut args = vec![
        "--newline".to_string(),
        "--no-update".to_string(),
        "--encoding".to_string(),
        "utf-8".to_string(),
    ];

    if settings.replace_unsafe_characters {
        args.push("--windows-filenames".to_string());
    }

    match job.download_type {
        DownloadType::AudioOnly => {
            args.push("-f".to_string());
            args.push("bestaudio/best".to_string());
            args.push("--extract-audio".to_string());
            args.push("--audio-format".to_string());
            args.push(job.audio_format.to_lowercase());
            args.push("--audio-quality".to_string());
            args.push(audio_quality_arg(&preset.audio_quality).to_string());
            if !settings.keep_original {
                args.push("--no-keep-video".to_string());
            }
        }
        DownloadType::VideoOnly => {
            args.push("-f".to_string());
            args.push("bestvideo".to_string());
        }
        DownloadType::FullVideo => {
            args.push("-f".to_string());
            args.push(preset.format_rule.clone());
            args.push("--merge-output-format".to_string());
            args.push(job.container.to_lowercase());
        }
    }

    if settings.write_subtitles {
        args.push("--write-subs".to_string());
        args.push("--sub-langs".to_string());
        args.push(settings.subtitle_languages.clone());
    }
    if settings.write_auto_subtitles {
        args.push("--write-auto-subs".to_string());
    }
    if settings.write_thumbnail {
        args.push("--write-thumbnail".to_string());
    }
    if settings.embed_thumbnail {
        args.push("--embed-thumbnail".to_string());
    }
    if settings.add_metadata {
        args.push("--add-metadata".to_string());
    }
    if settings.split_chapters {
        args.push("--split-chapters".to_string());
    }
    if settings.prevent_overwrites {
        args.push("--no-overwrites".to_string());
    }
    if settings.skip_existing {
        args.push("--continue".to_string());
    }

    args.push("--retries".to_string());
    args.push(settings.retries.to_string());
    args.push("--concurrent-fragments".to_string());
    args.push(settings.concurrent_fragments.to_string());

    if settings.speed_limit != "Unlimited" {
        args.push("--limit-rate".to_string());
        args.push(settings.speed_limit.clone());
    }

    add_common_network_args(&mut args, settings);
    args.push("-P".to_string());
    args.push(job.output_folder.clone());
    args.push("-o".to_string());
    args.push(job.output_template.clone());
    args.push(job.source_url.clone());
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
        .unwrap_or_default()
        .to_string();
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

pub(crate) fn parse_urls(input: &str) -> Vec<String> {
    input
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToString::to_string)
        .collect()
}

pub(crate) fn format_rule(format: &str, container: &str, codec: &str, cap: &str) -> String {
    match format {
        "Best quality" => "bestvideo+bestaudio/best".to_string(),
        "MP4 720p" => {
            "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720]".to_string()
        }
        "4K if available" => "bestvideo[height<=2160]+bestaudio/best[height<=2160]".to_string(),
        "Video only" => "bestvideo".to_string(),
        _ if container.eq_ignore_ascii_case("webm") => {
            format!("bestvideo[height<={}] + bestaudio/best", cap_to_number(cap))
        }
        _ if codec.eq_ignore_ascii_case("AV1") => {
            format!(
                "bestvideo[height<={}][vcodec*=av01]+bestaudio/best",
                cap_to_number(cap)
            )
        }
        _ => "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]".to_string(),
    }
}

pub(crate) fn queue_command_preview(ctx: FetchContext) -> String {
    let settings = ctx.settings();
    let preset = ctx.active_preset();
    let fake_item = MediaItem {
        title: i18n::t("selected_media"),
        uploader: String::new(),
        url: parse_urls(&(ctx.url_text)())
            .first()
            .cloned()
            .unwrap_or_else(|| "https://example.com/video".to_string()),
        duration: String::new(),
        thumbnail: String::new(),
        format_count: 0,
        estimated_size: String::new(),
        selected: true,
    };
    let fake_job = build_job(
        0,
        fake_item,
        (ctx.download_type)(),
        &(ctx.selected_format)(),
        &(ctx.selected_audio_format)(),
        &(ctx.container)(),
        &settings,
        &preset,
    );
    fake_job.command_display
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

pub(crate) fn path_to_arg(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

pub(crate) fn bundle_relative_display(path: &Path) -> String {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    path.strip_prefix(manifest_dir)
        .map(path_to_arg)
        .unwrap_or_else(|_| path_to_arg(path))
}

pub(crate) fn missing_binary_detail(path: &Path, detail: &str) -> String {
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

pub(crate) fn dependency_install_failed_detail(path: &Path, error: &AppError) -> String {
    i18n::t_with(
        "dependency_install_failed_detail",
        &[("path", path_to_arg(path)), ("error", error.debug.clone())],
    )
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
