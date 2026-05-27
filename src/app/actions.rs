use std::path::Path;

use super::*;

#[cfg(feature = "desktop")]
pub(crate) fn import_url_list(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(i18n::t("text_files"), &["txt", "csv", "list"])
        .pick_file()
    {
        match std::fs::read_to_string(&path) {
            Ok(content) => ctx.url_text.set(content),
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

pub(crate) fn pick_output_folder(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new().pick_folder() {
        ctx.settings
            .with_mut(|settings| settings.output_folder = path.display().to_string());
    }
}

pub(crate) fn pick_cookie_file(mut ctx: FetchContext) {
    if let Some(path) = rfd::FileDialog::new()
        .add_filter(i18n::t("cookie_files"), &["txt", "cookies"])
        .pick_file()
    {
        ctx.settings
            .with_mut(|settings| settings.cookie_file = path.display().to_string());
    }
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
            let job = build_job(
                id,
                item,
                download_type,
                &format_label,
                &audio_format,
                &container,
                &settings,
                &preset,
            );
            jobs.push(job);
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
    let mut job = DownloadJob {
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
        command_display: String::new(),
        status: JobStatus::Queued,
        progress: 0.0,
        speed: "-".to_string(),
        eta: "-".to_string(),
        step: "Queued".to_string(),
        output_hint: preview_output_path(settings),
        log: Vec::new(),
        error: None,
    };
    let args = build_download_args(&job, settings, preset);
    job.command_display = yt_dlp_command_display(&args);
    job
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

pub(crate) fn toggle_analysis_item(mut ctx: FetchContext, index: usize) {
    ctx.analysis.with_mut(|analysis| {
        if let Some(analysis) = analysis {
            if let Some(item) = analysis.items.get_mut(index) {
                item.selected = !item.selected;
            }
        }
    });
}

pub(crate) fn clear_completed(mut ctx: FetchContext) {
    ctx.jobs
        .with_mut(|jobs| jobs.retain(|job| job.status != JobStatus::Completed));
}

pub(crate) fn clear_history(mut ctx: FetchContext) {
    ctx.jobs.with_mut(Vec::clear);
}

pub(crate) fn copy_command_to_input(mut ctx: FetchContext, command: String) {
    ctx.url_text.set(command);
    ctx.screen.set(Screen::Home);
}

pub(crate) fn cancel_analysis(mut ctx: FetchContext) {
    ctx.analysis_cancel_token.with_mut(|token| *token += 1);
    ctx.analysis_running.set(false);
    ctx.busy.set(false);
}

pub(crate) fn open_ready_view(mut ctx: FetchContext) {
    ctx.show_cli.set(false);
    ctx.screen.set(Screen::Ready);
}

pub(crate) fn requeue_job(mut ctx: FetchContext, mut job: DownloadJob) {
    let id = (ctx.next_job_id)();
    ctx.next_job_id.set(id + 1);
    job.id = id;
    job.status = JobStatus::Queued;
    job.progress = 0.0;
    job.step = "Queued".to_string();
    job.error = None;
    job.log.clear();
    ctx.jobs.with_mut(|jobs| jobs.push(job));
    ctx.screen.set(Screen::Queue);
}

pub(crate) fn duplicate_preset(mut ctx: FetchContext) {
    let mut preset = ctx.active_preset();
    preset.name.push_str(" copy");
    ctx.presets.with_mut(|presets| {
        presets.push(preset);
        ctx.active_preset.set(presets.len() - 1);
    });
    persist_preset_store(ctx);
}

pub(crate) fn apply_preset(mut ctx: FetchContext) {
    let preset = ctx.active_preset();
    ctx.download_type.set(preset.kind);
    ctx.selected_format.set(preset.format_label.clone());
    ctx.selected_audio_format
        .set(preset.audio_format.to_uppercase());
    ctx.container.set(preset.container.to_uppercase());
    ctx.settings
        .with_mut(|settings| settings.file_template = preset.output_template);
}

pub(crate) fn update_selected_preset(mut ctx: FetchContext, field: &str, value: String) {
    ctx.presets.with_mut(|presets| {
        if let Some(preset) = presets.get_mut((ctx.active_preset)()) {
            match field {
                "name" => preset.name = value,
                "format_rule" => preset.format_rule = value,
                "audio_format" => preset.audio_format = value,
                "output_template" => preset.output_template = value,
                "extra_flags" => preset.extra_flags = value,
                _ => {}
            }
        }
    });
    persist_preset_store(ctx);
}

pub(crate) fn set_bool_setting(settings: &mut AppSettings, field: &str, value: bool) {
    match field {
        "write_subtitles" => settings.write_subtitles = value,
        "write_auto_subtitles" => settings.write_auto_subtitles = value,
        "write_thumbnail" => settings.write_thumbnail = value,
        "embed_thumbnail" => settings.embed_thumbnail = value,
        "add_metadata" => settings.add_metadata = value,
        "split_chapters" => settings.split_chapters = value,
        "keep_original" => settings.keep_original = value,
        "create_playlist_folders" => settings.create_playlist_folders = value,
        "add_playlist_index" => settings.add_playlist_index = value,
        "replace_unsafe_characters" => settings.replace_unsafe_characters = value,
        "prevent_overwrites" => settings.prevent_overwrites = value,
        "skip_existing" => settings.skip_existing = value,
        _ => {}
    }
}

pub(crate) fn set_i32_setting(settings: &mut AppSettings, field: &str, value: i32) {
    match field {
        "parallel_jobs" => settings.parallel_jobs = value,
        "concurrent_fragments" => settings.concurrent_fragments = value,
        "retries" => settings.retries = value,
        _ => {}
    }
}

pub(crate) fn reveal_output(path: &str) {
    let path = Path::new(path);
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
