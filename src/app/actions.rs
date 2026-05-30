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
    let preset = ctx.active_preset();
    let download_type = (ctx.download_type)();
    let format_label = (ctx.selected_format)();
    let audio_format = (ctx.selected_audio_format)();
    let container = (ctx.container)();

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
                &container,
                &settings,
                &preset,
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
    container: &str,
    settings: &AppSettings,
    preset: &Preset,
) -> DownloadJob {
    let command_display = queue_command_display(
        &item.url,
        download_type,
        format_label,
        audio_format,
        container,
        settings,
        preset,
    );

    DownloadJob {
        id,
        title: item.title,
        source_url: item.url,
        thumbnail: item.thumbnail,
        download_type,
        format_label: format_label.to_string(),
        audio_format: audio_format.to_lowercase(),
        container: container.to_lowercase(),
        output_folder: settings.output_folder.clone(),
        output_template: settings.file_template.clone(),
        command_display,
        status: JobStatus::Queued,
        progress: 0.0,
        speed: "-".to_string(),
        eta: "-".to_string(),
        step: i18n::t("job_step_queued"),
        output_hint: settings.output_folder.clone(),
        log: Vec::new(),
        error: None,
    }
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
    let preset = ctx.active_preset();
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
        &(ctx.container)(),
        &settings,
        &preset,
    )
}

pub(crate) fn queue_command_display(
    source_url: &str,
    download_type: DownloadType,
    format_label: &str,
    audio_format: &str,
    container: &str,
    settings: &AppSettings,
    preset: &Preset,
) -> String {
    yt_dlp_command_display(&queue_command_args(
        source_url,
        download_type,
        format_label,
        audio_format,
        container,
        settings,
        preset,
    ))
}

fn queue_command_args(
    source_url: &str,
    download_type: DownloadType,
    _format_label: &str,
    audio_format: &str,
    container: &str,
    settings: &AppSettings,
    preset: &Preset,
) -> Vec<String> {
    let mut args = vec![
        "--newline".to_string(),
        "--no-update".to_string(),
        "--encoding".to_string(),
        "utf-8".to_string(),
    ];

    match download_type {
        DownloadType::AudioOnly => {
            args.push("-f".to_string());
            args.push("bestaudio/best".to_string());
            args.push("--extract-audio".to_string());
            args.push("--audio-format".to_string());
            args.push(audio_format.to_lowercase());
        }
        DownloadType::VideoOnly => {
            args.push("-f".to_string());
            args.push("bestvideo".to_string());
        }
        DownloadType::FullVideo => {
            args.push("-f".to_string());
            args.push(preset.format_rule.clone());
            args.push("--merge-output-format".to_string());
            args.push(container.to_lowercase());
        }
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
        let preset = Preset::defaults().remove(0);

        let args = queue_command_args(
            "https://example.com/video",
            DownloadType::FullVideo,
            "MP4 1080p",
            "MP3",
            "MP4",
            &settings,
            &preset,
        );

        assert!(args.contains(&"-f".to_string()));
        assert!(args.contains(&preset.format_rule));
        assert!(args.contains(&"--merge-output-format".to_string()));
        assert!(args.contains(&"mp4".to_string()));
        assert!(args.contains(&"-P".to_string()));
        assert!(args.contains(&settings.output_folder));
        assert_eq!(args.last(), Some(&"https://example.com/video".to_string()));
    }

    #[test]
    fn queue_command_args_support_audio_mode() {
        let settings = AppSettings::default();
        let preset = Preset::defaults().remove(1);

        let args = queue_command_args(
            "https://example.com/audio",
            DownloadType::AudioOnly,
            "Best audio",
            "M4A",
            "MP4",
            &settings,
            &preset,
        );

        assert!(args.contains(&"--extract-audio".to_string()));
        assert!(args.contains(&"--audio-format".to_string()));
        assert!(args.contains(&"m4a".to_string()));
        assert_eq!(args.last(), Some(&"https://example.com/audio".to_string()));
    }
}
