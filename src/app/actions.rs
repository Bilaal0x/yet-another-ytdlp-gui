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
            let candidate = normalize_url_candidate(&candidate);
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
    let command_args = queue_command_args(
        &item.url,
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
        source_url: item.url,
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
    let url = (ctx.analysis)()
        .and_then(|analysis| {
            analysis
                .items
                .iter()
                .find(|item| item.selected)
                .or_else(|| analysis.items.first())
                .map(|item| item.url.clone())
        })
        .or_else(|| parse_urls(&(ctx.url_text)()).first().cloned())
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
    yt_dlp_command_display(&queue_command_args(
        source_url,
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
    let mut args = vec![
        "--newline".to_string(),
        "--no-update".to_string(),
        "--encoding".to_string(),
        "utf-8".to_string(),
    ];
    add_common_network_args(&mut args, settings);

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
    args.push(source_url.to_string());

    args
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
        assert!(args.contains(&"--audio-format".to_string()));
        assert!(args.contains(&"m4a".to_string()));
        assert!(args.contains(&"--audio-quality".to_string()));
        assert!(args.contains(&"320K".to_string()));
        assert_eq!(args.last(), Some(&"https://example.com/audio".to_string()));
    }

    #[test]
    fn build_download_args_adds_runtime_controls_before_url() {
        let mut settings = AppSettings::default();
        settings.retries = 7;
        settings.concurrent_fragments = 3;
        settings.speed_limit = "5M".to_string();
        let item = MediaItem {
            title: "Video".to_string(),
            uploader: "Uploader".to_string(),
            url: "https://example.com/video".to_string(),
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
