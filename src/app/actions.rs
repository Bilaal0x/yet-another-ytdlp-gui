use super::*;

#[derive(Clone, PartialEq, Eq)]
pub(crate) struct UrlIntake {
    pub(crate) urls: Vec<String>,
    pub(crate) invalid_lines: Vec<String>,
}

impl UrlIntake {
    pub(crate) fn can_analyze(&self) -> bool {
        !self.urls.is_empty() && self.invalid_lines.is_empty()
    }
}

pub(crate) fn parse_urls(input: &str) -> Vec<String> {
    parse_url_intake(input).urls
}

pub(crate) fn parse_url_intake(input: &str) -> UrlIntake {
    let mut urls = Vec::new();
    let mut invalid_lines = Vec::new();

    for raw_line in input.lines() {
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let cells = split_list_cells(line);
        let mut found_url = false;
        for candidate in &cells {
            let candidate = normalize_url_candidate(candidate);
            if candidate.is_empty() {
                continue;
            }

            if is_valid_url(&candidate) {
                urls.push(candidate);
                found_url = true;
            }
        }

        if !found_url && !is_header_row(&cells) {
            invalid_lines.push(line.to_string());
        }
    }

    UrlIntake {
        urls,
        invalid_lines,
    }
}

pub(crate) fn start_analysis(mut ctx: FetchContext) {
    let intake = parse_url_intake(&(ctx.url_text)());

    if intake.urls.is_empty() {
        ctx.last_error.set(Some(AppError::new(
            i18n::t("error_no_url_title"),
            i18n::t("error_no_url_message"),
            "",
        )));
        return;
    }

    if !intake.invalid_lines.is_empty() {
        ctx.last_error.set(Some(AppError::new(
            i18n::t("error_invalid_url_title"),
            i18n::t_with(
                "error_invalid_url_message",
                &[("count", intake.invalid_lines.len().to_string())],
            ),
            intake.invalid_lines.join("\n"),
        )));
        return;
    }

    ctx.analysis_cancel_token.with_mut(|token| *token += 1);
    let cancel_generation = (ctx.analysis_cancel_token)();
    ctx.analysis.set(None);
    ctx.last_error.set(None);
    ctx.busy.set(true);
    ctx.analysis_running.set(true);
    ctx.backend.send(BackendAction::Analyze {
        urls: intake.urls,
        settings: (ctx.settings)(),
        cancel_generation,
    });
}

pub(crate) fn cancel_analysis(mut ctx: FetchContext) {
    ctx.analysis_cancel_token.with_mut(|token| *token += 1);
    ctx.analysis_running.set(false);
    ctx.busy.set(false);
}

pub(crate) fn add_analysis_to_queue(mut ctx: FetchContext) {
    let Some(analysis) = (ctx.analysis)() else {
        ctx.last_error.set(Some(AppError::new(
            i18n::t("nothing_to_queue"),
            i18n::t("analyze_before_queue"),
            "",
        )));
        ctx.screen.set(Screen::Error);
        return;
    };

    let selected: Vec<MediaItem> = analysis
        .items
        .into_iter()
        .filter(|item| item.selected)
        .collect();

    if selected.is_empty() {
        ctx.last_error.set(Some(AppError::new(
            i18n::t("nothing_selected"),
            i18n::t("select_before_queue"),
            "",
        )));
        ctx.screen.set(Screen::Error);
        return;
    }

    let settings = ctx.settings();
    let download_type = (ctx.download_type)();
    let format_label = (ctx.selected_format)();
    let audio_format = (ctx.selected_audio_format)();
    let audio_quality = (ctx.audio_quality)();
    let container = (ctx.container)();
    let video_codec = (ctx.video_codec)();
    let resolution_cap = (ctx.resolution_cap)();

    ctx.jobs.with_mut(|jobs| {
        for item in selected {
            let id = (ctx.next_job_id)();
            ctx.next_job_id.set(id + 1);
            jobs.push(build_job(
                id,
                item,
                download_type,
                &format_label,
                &audio_format,
                &audio_quality,
                &container,
                &video_codec,
                &resolution_cap,
                &settings,
            ));
        }
    });
    ctx.screen.set(Screen::Queue);
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_job(
    id: u64,
    item: MediaItem,
    download_type: DownloadType,
    format_label: &str,
    audio_format: &str,
    audio_quality: &str,
    container: &str,
    video_codec: &str,
    resolution_cap: &str,
    settings: &AppSettings,
) -> DownloadJob {
    let queue_source = item.queue_source();
    let command_args = queue_command_args_for_source(
        &queue_source,
        download_type,
        format_label,
        audio_format,
        audio_quality,
        container,
        video_codec,
        resolution_cap,
        settings,
    );
    let command_display = yt_dlp_command_display(&command_args);

    DownloadJob {
        id,
        title: item.title,
        source_url: queue_source.url().to_string(),
        thumbnail: item.thumbnail,
        download_type,
        format_label: format_label.to_string(),
        audio_format: audio_format.to_lowercase(),
        audio_quality: audio_quality_arg(audio_quality).to_string(),
        container: container.to_lowercase(),
        video_codec: video_codec.to_string(),
        resolution_cap: resolution_cap.to_string(),
        output_folder: settings.output_folder.clone(),
        output_template: settings.file_template.clone(),
        command_args,
        command_display,
        status: JobStatus::Queued,
        progress: 0.0,
        speed: "-".to_string(),
        eta: "-".to_string(),
        step: i18n::t("job_step_queued"),
        output_hint: preview_output_path(settings),
        log: Vec::new(),
        error: None,
    }
}

pub(crate) fn clear_completed(mut ctx: FetchContext) {
    ctx.jobs
        .with_mut(|jobs| jobs.retain(|job| job.status != JobStatus::Completed));
}

pub(crate) fn clear_history(mut ctx: FetchContext) {
    ctx.jobs.with_mut(Vec::clear);
}

pub(crate) fn forget_cookie_file(mut ctx: FetchContext) {
    ctx.settings
        .with_mut(|settings| settings.cookie_file.clear());
}

pub(crate) fn select_preset(mut ctx: FetchContext, index: usize) {
    if (ctx.presets)().get(index).is_none() {
        return;
    }

    ctx.active_preset.set(index);
    apply_preset(ctx);
    persist_preset_store(ctx);
}

pub(crate) fn duplicate_preset(mut ctx: FetchContext) {
    let mut preset = ctx.active_preset();
    preset.name = format!("{} copy", preset.name);

    ctx.presets.with_mut(|presets| {
        presets.push(preset);
        ctx.active_preset.set(presets.len() - 1);
    });
    apply_preset(ctx);
    persist_preset_store(ctx);
}

pub(crate) fn apply_preset(mut ctx: FetchContext) {
    let preset = ctx.active_preset();
    ctx.download_type.set(preset.kind);
    ctx.selected_format.set(preset.format_label.clone());
    ctx.selected_audio_format
        .set(preset.audio_format.to_uppercase());
    ctx.audio_quality
        .set(audio_quality_label(&preset.audio_quality));
    ctx.container.set(preset.container.to_uppercase());
    ctx.video_codec
        .set(default_video_codec(&preset.format_label).to_string());
    ctx.resolution_cap
        .set(default_resolution_cap(&preset.format_label).to_string());
    ctx.settings
        .with_mut(|settings| settings.file_template = preset.output_template);
}

pub(crate) fn update_selected_preset(mut ctx: FetchContext, field: &str, value: String) {
    ctx.presets.with_mut(|presets| {
        if let Some(preset) = presets.get_mut((ctx.active_preset)()) {
            set_preset_field(preset, field, value);
        }
    });
    persist_preset_store(ctx);
}

pub(crate) fn update_selected_preset_kind(mut ctx: FetchContext, kind: DownloadType) {
    ctx.presets.with_mut(|presets| {
        if let Some(preset) = presets.get_mut((ctx.active_preset)()) {
            preset.kind = kind;
        }
    });
    persist_preset_store(ctx);
}

pub(crate) fn requeue_job(mut ctx: FetchContext, mut job: DownloadJob) {
    let id = (ctx.next_job_id)();
    ctx.next_job_id.set(id + 1);
    job.id = id;
    job.status = JobStatus::Queued;
    job.progress = 0.0;
    job.speed = "-".to_string();
    job.eta = "-".to_string();
    job.step = i18n::t("job_step_queued");
    job.error = None;
    job.log.clear();
    ctx.jobs.with_mut(|jobs| jobs.push(job));
    ctx.screen.set(Screen::Queue);
}

pub(crate) fn set_all_analysis_items(mut ctx: FetchContext, selected: bool) {
    ctx.analysis.with_mut(|analysis| {
        if let Some(analysis) = analysis {
            for item in &mut analysis.items {
                item.selected = selected;
            }
        }
    });
}

pub(crate) fn select_first_n(mut ctx: FetchContext, count: usize) {
    ctx.analysis.with_mut(|analysis| {
        if let Some(analysis) = analysis {
            for (index, item) in analysis.items.iter_mut().enumerate() {
                item.selected = index < count;
            }
        }
    });
}

pub(crate) fn queue_command_preview(ctx: FetchContext) -> String {
    let settings = ctx.settings();
    if let Some(source) = (ctx.analysis)().and_then(|analysis| {
        analysis
            .items
            .iter()
            .find(|item| item.selected)
            .or_else(|| analysis.items.first())
            .map(MediaItem::queue_source)
    }) {
        return queue_command_display_for_source(
            &source,
            (ctx.download_type)(),
            &(ctx.selected_format)(),
            &(ctx.selected_audio_format)(),
            &(ctx.audio_quality)(),
            &(ctx.container)(),
            &(ctx.video_codec)(),
            &(ctx.resolution_cap)(),
            &settings,
        );
    }

    let url = parse_urls(&(ctx.url_text)())
        .first()
        .cloned()
        .unwrap_or_else(|| "https://example.com/video".to_string());

    queue_command_display(
        &url,
        (ctx.download_type)(),
        &(ctx.selected_format)(),
        &(ctx.selected_audio_format)(),
        &(ctx.audio_quality)(),
        &(ctx.container)(),
        &(ctx.video_codec)(),
        &(ctx.resolution_cap)(),
        &settings,
    )
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn queue_command_display(
    source_url: &str,
    download_type: DownloadType,
    format_label: &str,
    audio_format: &str,
    audio_quality: &str,
    container: &str,
    video_codec: &str,
    resolution_cap: &str,
    settings: &AppSettings,
) -> String {
    queue_command_display_for_source(
        &QueueSource::Direct {
            url: source_url.to_string(),
        },
        download_type,
        format_label,
        audio_format,
        audio_quality,
        container,
        video_codec,
        resolution_cap,
        settings,
    )
}

pub(crate) fn build_download_args(job: &DownloadJob, settings: &AppSettings) -> Vec<String> {
    let mut args = job.command_args.clone();
    let source_url = args.pop();

    args.push("--retries".to_string());
    args.push(settings.retries.max(0).to_string());
    args.push("--concurrent-fragments".to_string());
    args.push(settings.concurrent_fragments.max(1).to_string());

    if settings.speed_limit != "Unlimited" {
        args.push("--limit-rate".to_string());
        args.push(settings.speed_limit.clone());
    }

    if let Some(source_url) = source_url {
        args.push(source_url);
    }

    args
}

fn set_preset_field(preset: &mut Preset, field: &str, value: String) {
    match field {
        "name" => preset.name = value,
        "format_label" => preset.format_label = value,
        "format_rule" => preset.format_rule = value,
        "audio_format" => preset.audio_format = value,
        "audio_quality" => preset.audio_quality = value,
        "container" => preset.container = value,
        "output_template" => preset.output_template = value,
        "extra_flags" => preset.extra_flags = value,
        _ => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn queue_command_args(
    source_url: &str,
    download_type: DownloadType,
    format_label: &str,
    audio_format: &str,
    audio_quality: &str,
    container: &str,
    video_codec: &str,
    resolution_cap: &str,
    settings: &AppSettings,
) -> Vec<String> {
    queue_command_args_for_source(
        &QueueSource::Direct {
            url: source_url.to_string(),
        },
        download_type,
        format_label,
        audio_format,
        audio_quality,
        container,
        video_codec,
        resolution_cap,
        settings,
    )
}

#[allow(clippy::too_many_arguments)]
fn queue_command_display_for_source(
    source: &QueueSource,
    download_type: DownloadType,
    format_label: &str,
    audio_format: &str,
    audio_quality: &str,
    container: &str,
    video_codec: &str,
    resolution_cap: &str,
    settings: &AppSettings,
) -> String {
    yt_dlp_command_display(&queue_command_args_for_source(
        source,
        download_type,
        format_label,
        audio_format,
        audio_quality,
        container,
        video_codec,
        resolution_cap,
        settings,
    ))
}

#[allow(clippy::too_many_arguments)]
fn queue_command_args_for_source(
    source: &QueueSource,
    download_type: DownloadType,
    format_label: &str,
    audio_format: &str,
    audio_quality: &str,
    container: &str,
    video_codec: &str,
    resolution_cap: &str,
    settings: &AppSettings,
) -> Vec<String> {
    let mut args = vec![
        "--newline".to_string(),
        "--no-update".to_string(),
        "--encoding".to_string(),
        "utf-8".to_string(),
    ];
    add_common_network_args(&mut args, settings);
    add_queue_source_args(&mut args, source);

    match download_type {
        DownloadType::AudioOnly => {
            args.push("-f".to_string());
            args.push("bestaudio/best".to_string());
            args.push("--extract-audio".to_string());
            args.push("--audio-format".to_string());
            args.push(audio_format.to_lowercase());
            args.push("--audio-quality".to_string());
            args.push(audio_quality_arg(audio_quality).to_string());
            if settings.keep_original {
                args.push("--keep-video".to_string());
            }
        }
        DownloadType::VideoOnly => {
            args.push("-f".to_string());
            args.push(format_rule(
                "Video only",
                container,
                video_codec,
                resolution_cap,
            ));
        }
        DownloadType::FullVideo => {
            args.push("-f".to_string());
            args.push(format_rule(
                format_label,
                container,
                video_codec,
                resolution_cap,
            ));
            args.push("--merge-output-format".to_string());
            args.push(container.to_lowercase());
        }
    }

    if settings.embed_thumbnail {
        args.push("--embed-thumbnail".to_string());
    }
    if settings.write_thumbnail {
        args.push("--write-thumbnail".to_string());
    }
    if settings.add_metadata {
        args.push("--add-metadata".to_string());
    }
    if settings.write_subtitles {
        args.push("--write-subs".to_string());
        args.push("--sub-langs".to_string());
        args.push(settings.subtitle_languages.clone());
    }
    if settings.write_auto_subtitles {
        args.push("--write-auto-subs".to_string());
    }
    if settings.split_chapters {
        args.push("--split-chapters".to_string());
    }
    if settings.replace_unsafe_characters {
        args.push("--windows-filenames".to_string());
    }
    if settings.prevent_overwrites {
        args.push("--no-overwrites".to_string());
    }
    if settings.skip_existing {
        args.push("--continue".to_string());
    }

    args.push("-P".to_string());
    args.push(settings.output_folder.clone());
    args.push("-o".to_string());
    args.push(settings.file_template.clone());
    args.push(source.url().to_string());

    args
}

fn add_queue_source_args(args: &mut Vec<String>, source: &QueueSource) {
    match source {
        QueueSource::Direct { .. } => {
            args.push("--no-playlist".to_string());
        }
        QueueSource::PlaylistItem { index, .. } => {
            args.push("--playlist-items".to_string());
            args.push(index.to_string());
        }
    }
}

#[cfg(feature = "desktop")]
pub(crate) fn import_url_list(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(i18n::t("text_files"), &["txt", "csv", "list"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(content) => {
                ctx.url_text.set(content);
                ctx.last_error.set(None);
            }
            Err(error) => {
                ctx.last_error.set(Some(AppError::new(
                    i18n::t("import_failed"),
                    i18n::t_with(
                        "could_not_read_file",
                        &[("path", path.display().to_string())],
                    ),
                    error.to_string(),
                )));
                ctx.screen.set(Screen::Error);
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
pub(crate) fn import_url_list(_ctx: FetchContext) {}

#[cfg(feature = "desktop")]
pub(crate) fn pick_output_folder(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        ctx.settings
            .with_mut(|settings| settings.output_folder = path.display().to_string());
    }
}

#[cfg(not(feature = "desktop"))]
pub(crate) fn pick_output_folder(_ctx: FetchContext) {}

#[cfg(feature = "desktop")]
pub(crate) fn pick_cookie_file(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(i18n::t("cookie_files"), &["txt", "cookies"])
        .pick_file()
    {
        ctx.settings
            .with_mut(|settings| settings.cookie_file = path.display().to_string());
    }
}

#[cfg(not(feature = "desktop"))]
pub(crate) fn pick_cookie_file(_ctx: FetchContext) {}

pub(crate) fn reveal_output(path: &str) {
    let path = std::path::Path::new(path);

    #[cfg(target_os = "windows")]
    {
        let target = if path.exists() {
            path.to_path_buf()
        } else {
            path.parent().unwrap_or(path).to_path_buf()
        };
        let _ = std::process::Command::new("explorer").arg(target).spawn();
    }

    #[cfg(target_os = "macos")]
    {
        let target = path.parent().unwrap_or(path);
        let _ = std::process::Command::new("open").arg(target).spawn();
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let target = path.parent().unwrap_or(path);
        let _ = std::process::Command::new("xdg-open").arg(target).spawn();
    }
}

pub(crate) fn open_ready_view(mut ctx: FetchContext) {
    if (ctx.analysis)().is_some() {
        ctx.screen.set(Screen::Ready);
    } else {
        ctx.screen.set(Screen::Home);
    }
}

fn split_list_cells(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    let mut quoted = false;

    while let Some(ch) = chars.next() {
        match ch {
            '"' if quoted && chars.peek() == Some(&'"') => {
                current.push('"');
                chars.next();
            }
            '"' => quoted = !quoted,
            ',' | ';' | '\t' if !quoted => {
                cells.push(current.trim().to_string());
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    cells.push(current.trim().to_string());
    cells
}

fn normalize_url_candidate(value: &str) -> String {
    let mut candidate = value.trim();

    if candidate.starts_with('<') && candidate.ends_with('>') && candidate.len() > 2 {
        candidate = &candidate[1..candidate.len() - 1];
    }

    candidate.trim().to_string()
}

fn is_valid_url(value: &str) -> bool {
    if value.chars().any(char::is_whitespace) {
        return false;
    }

    let lower = value.to_ascii_lowercase();
    let Some(rest) = lower
        .strip_prefix("https://")
        .or_else(|| lower.strip_prefix("http://"))
    else {
        return false;
    };

    let host = rest.split('/').next().unwrap_or_default();
    !host.is_empty() && host.contains('.')
}

fn is_header_row(cells: &[String]) -> bool {
    cells.iter().any(|cell| {
        matches!(
            cell.trim().to_ascii_lowercase().as_str(),
            "url" | "urls" | "link" | "links" | "source" | "source_url" | "webpage_url"
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_one_url_per_line_and_ignores_comments() {
        let intake = parse_url_intake(
            r#"
            # playlist links
            https://example.com/watch?v=one

            https://youtu.be/two
            "#,
        );

        assert_eq!(
            intake.urls,
            vec![
                "https://example.com/watch?v=one".to_string(),
                "https://youtu.be/two".to_string()
            ]
        );
        assert!(intake.invalid_lines.is_empty());
    }

    #[test]
    fn extracts_urls_from_csv_cells() {
        let intake = parse_url_intake(
            r#"
            title,url
            "First","https://example.com/one"
            Second;https://example.com/two
            "#,
        );

        assert_eq!(
            intake.urls,
            vec![
                "https://example.com/one".to_string(),
                "https://example.com/two".to_string()
            ]
        );
        assert!(intake.invalid_lines.is_empty());
    }

    #[test]
    fn reports_lines_without_supported_urls() {
        let intake = parse_url_intake(
            r#"
            www.example.com/no-scheme
            notes only
            https://example.com/ok
            "#,
        );

        assert_eq!(intake.urls, vec!["https://example.com/ok".to_string()]);
        assert_eq!(
            intake.invalid_lines,
            vec![
                "www.example.com/no-scheme".to_string(),
                "notes only".to_string()
            ]
        );
    }

    #[test]
    fn queue_command_args_include_mode_and_output() {
        let settings = AppSettings::default();
        let args = queue_command_args(
            "https://example.com/video",
            DownloadType::FullVideo,
            "MP4 1080p",
            "MP3",
            "320 kbps",
            "MP4",
            "H.264",
            "1080p",
            &settings,
        );

        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(
            &"bestvideo[height<=1080][vcodec^=avc1][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]"
                .to_string()
        ));
        assert!(args.contains(&"--merge-output-format".to_string()));
        assert!(args.contains(&"mp4".to_string()));
        assert!(args.contains(&"--no-playlist".to_string()));
        assert!(args.contains(&"-P".to_string()));
        assert!(args.contains(&settings.output_folder));
        assert_eq!(args.last(), Some(&"https://example.com/video".to_string()));
    }

    #[test]
    fn queue_command_args_support_audio_mode() {
        let settings = AppSettings::default();
        let args = queue_command_args(
            "https://example.com/audio",
            DownloadType::AudioOnly,
            "Best audio",
            "M4A",
            "320 kbps",
            "MP4",
            "H.264",
            "1080p",
            &settings,
        );

        assert!(args.contains(&"--extract-audio".to_string()));
        assert!(args.contains(&"--no-playlist".to_string()));
        assert!(args.contains(&"--audio-format".to_string()));
        assert!(args.contains(&"m4a".to_string()));
        assert!(args.contains(&"--audio-quality".to_string()));
        assert!(args.contains(&"320K".to_string()));
        assert_eq!(args.last(), Some(&"https://example.com/audio".to_string()));
    }

    #[test]
    fn queue_command_args_support_playlist_item_sources() {
        let settings = AppSettings {
            cookie_file: "cookies.txt".to_string(),
            proxy: "socks5://localhost:9000".to_string(),
            ..Default::default()
        };
        let source = QueueSource::PlaylistItem {
            playlist_url: "https://example.com/playlist".to_string(),
            index: 7,
        };

        let args = queue_command_args_for_source(
            &source,
            DownloadType::FullVideo,
            "MP4 1080p",
            "MP3",
            "320 kbps",
            "MP4",
            "H.264",
            "1080p",
            &settings,
        );

        assert!(args.contains(&"--cookies".to_string()));
        assert!(args.contains(&"cookies.txt".to_string()));
        assert!(args.contains(&"--proxy".to_string()));
        assert!(args.contains(&"socks5://localhost:9000".to_string()));
        assert!(args.contains(&"--playlist-items".to_string()));
        assert!(args.contains(&"7".to_string()));
        assert!(!args.contains(&"--no-playlist".to_string()));
        assert_eq!(
            args.last(),
            Some(&"https://example.com/playlist".to_string())
        );
    }

    #[test]
    fn build_job_uses_playlist_selector_for_playlist_items() {
        let settings = AppSettings::default();
        let item = MediaItem {
            title: "Playlist video".to_string(),
            uploader: "Uploader".to_string(),
            url: "https://example.com/watch".to_string(),
            entry_url: Some("https://example.com/watch".to_string()),
            playlist_url: Some("https://example.com/playlist".to_string()),
            playlist_index: Some(3),
            duration: "1:00".to_string(),
            thumbnail: String::new(),
            format_count: 1,
            estimated_size: "1 MB".to_string(),
            selected: true,
        };

        let job = build_job(
            1,
            item,
            DownloadType::FullVideo,
            "MP4 1080p",
            "MP3",
            "320 kbps",
            "MP4",
            "H.264",
            "1080p",
            &settings,
        );

        assert_eq!(job.source_url, "https://example.com/playlist");
        assert!(job.command_args.contains(&"--playlist-items".to_string()));
        assert!(job.command_args.contains(&"3".to_string()));
        assert!(!job.command_args.contains(&"--no-playlist".to_string()));
        assert_eq!(
            job.command_args.last(),
            Some(&"https://example.com/playlist".to_string())
        );
    }

    #[test]
    fn build_download_args_adds_runtime_controls_before_url() {
        let settings = AppSettings {
            retries: 7,
            concurrent_fragments: 3,
            speed_limit: "5M".to_string(),
            ..Default::default()
        };
        let item = MediaItem {
            title: "Video".to_string(),
            uploader: "Uploader".to_string(),
            url: "https://example.com/video".to_string(),
            entry_url: Some("https://example.com/video".to_string()),
            playlist_url: None,
            playlist_index: None,
            duration: "1:00".to_string(),
            thumbnail: String::new(),
            format_count: 1,
            estimated_size: "1 MB".to_string(),
            selected: true,
        };
        let job = build_job(
            1,
            item,
            DownloadType::FullVideo,
            "MP4 1080p",
            "MP3",
            "320 kbps",
            "MP4",
            "H.264",
            "1080p",
            &settings,
        );

        let args = build_download_args(&job, &settings);

        assert!(args.contains(&"--retries".to_string()));
        assert!(args.contains(&"7".to_string()));
        assert!(args.contains(&"--concurrent-fragments".to_string()));
        assert!(args.contains(&"3".to_string()));
        assert!(args.contains(&"--limit-rate".to_string()));
        assert!(args.contains(&"5M".to_string()));
        assert_eq!(args.last(), Some(&"https://example.com/video".to_string()));
    }
}
