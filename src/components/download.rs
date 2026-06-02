use super::super::*;
use super::StatBlock;

#[component]
pub(crate) fn PresetButton(index: usize, preset: Preset) -> Element {
    let ctx = use_context::<FetchContext>();
    let detail = match preset.kind {
        DownloadType::AudioOnly => format!(
            "{} {}",
            preset.audio_format.to_uppercase(),
            preset.audio_quality
        ),
        _ => format!(
            "{} / {}",
            preset.container.to_uppercase(),
            preset.format_label
        ),
    };

    rsx! {
        button {
            class: if (ctx.active_preset)() == index { "preset-row active" } else { "preset-row" },
            onclick: move |_| select_preset(ctx, index),
            span { class: "preset-name", "{preset.name}" }
            span { class: "preset-detail", "{detail}" }
        }
    }
}

#[component]
pub(crate) fn MediaPreview(item: MediaItem) -> Element {
    rsx! {
        div { class: "preview-card media-preview",
            ThumbnailBlock {
                class: "thumbnail-block large".to_string(),
                src: item.thumbnail.clone(),
                fallback: i18n::t("preview"),
            }
            div { class: "preview-body",
                h3 { "{item.title}" }
                div { class: "meta-grid",
                    span { "{i18n::t(\"duration\")}" }
                    strong { "{item.duration}" }
                    span { "{i18n::t(\"uploader\")}" }
                    strong { "{item.uploader}" }
                    span { "{i18n::t(\"source\")}" }
                    strong { "{item.url}" }
                    span { "{i18n::t(\"table_formats\")}" }
                    strong { "{item.format_count}" }
                    span { "{i18n::t(\"table_estimate\")}" }
                    strong { "{item.estimated_size}" }
                }
            }
        }
    }
}

#[component]
pub(crate) fn ModeButton(mode: DownloadType, label: String, detail: String) -> Element {
    let mut ctx = use_context::<FetchContext>();

    rsx! {
        button {
            class: if (ctx.download_type)() == mode { "mode-button active" } else { "mode-button" },
            onclick: move |_| {
                ctx.download_type.set(mode);
            },
            span { class: "mode-title", "{label}" }
            span { class: "mode-copy", "{detail}" }
        }
    }
}

#[component]
pub(crate) fn InspectorSettings() -> Element {
    let ctx = use_context::<FetchContext>();
    let settings = ctx.settings.read().clone();
    let subtitles = if settings.write_subtitles {
        settings.subtitle_languages
    } else {
        i18n::t("off")
    };
    let metadata = if settings.add_metadata {
        i18n::t("embedded")
    } else {
        i18n::t("off")
    };

    rsx! {
        div { class: "panel-heading compact",
            div { class: "eyebrow", "{i18n::t(\"quick_settings\")}" }
            h3 { "{i18n::t(\"download_setup\")}" }
        }
        StatBlock { label: i18n::t("quality"), value: (ctx.selected_format)() }
        StatBlock { label: i18n::t("container"), value: (ctx.container)() }
        StatBlock { label: i18n::t("output"), value: settings.output_folder }
        StatBlock { label: i18n::t("subtitles"), value: subtitles }
        StatBlock { label: i18n::t("metadata"), value: metadata }
    }
}

#[component]
pub(crate) fn PlaylistRow(index: usize, item: MediaItem) -> Element {
    let mut ctx = use_context::<FetchContext>();
    let row_class = if item.selected {
        "playlist-row selected"
    } else {
        "playlist-row muted"
    };

    rsx! {
        div { class: "{row_class}",
            input {
                r#type: "checkbox",
                checked: item.selected,
                onchange: move |_| {
                    ctx.analysis.with_mut(|analysis| {
                        if let Some(result) = analysis {
                            if let Some(item) = result.items.get_mut(index) {
                                item.selected = !item.selected;
                            }
                        }
                    });
                },
            }
            div { class: "video-cell",
                ThumbnailBlock {
                    class: "thumbnail-block tiny".to_string(),
                    src: item.thumbnail,
                    fallback: (index + 1).to_string(),
                }
                strong { "{item.title}" }
            }
            span { "{item.uploader}" }
            span { "{item.duration}" }
            span { "{item.format_count}" }
            span { "{item.estimated_size}" }
        }
    }
}

#[component]
pub(crate) fn DownloadRow(job: DownloadJob) -> Element {
    let progress = job.progress.clamp(0.0, 100.0);
    let class = format!("download-row {}", job.status.class());
    let log_open = job.status == JobStatus::Failed || job.status == JobStatus::Running;

    rsx! {
        div { class: "{class}",
            ThumbnailBlock {
                class: "thumbnail-block small".to_string(),
                src: job.thumbnail.clone(),
                fallback: job.status.label(),
            }
            div { class: "download-main",
                div { class: "download-titleline",
                    strong { "{job.title}" }
                    span { "{progress:.1}%" }
                }
                div { class: "progress-track",
                    div { class: "progress-fill", style: "width: {progress}%;" }
                }
                div { class: "download-meta",
                    span { "{job.step}" }
                    span { "{job.speed}" }
                    span { "{i18n::t(\"eta\")} {job.eta}" }
                    span { "{job.output_hint}" }
                }
                if log_open {
                    pre { class: "job-log", "{job.log.join(\"\\n\")}" }
                }
                div { class: "card-actions",
                    button { "{i18n::t(\"use_command\")}" }
                    if job.status == JobStatus::Completed {
                        button { "{i18n::t(\"reveal\")}" }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn LibraryCard(job: DownloadJob) -> Element {
    let ctx = use_context::<FetchContext>();
    let format_summary = library_format_summary(&job);
    let output_hint = job.output_hint.clone();
    let requeue = job.clone();

    rsx! {
        div { class: "library-card",
            ThumbnailBlock {
                class: "thumbnail-block library-thumb".to_string(),
                src: job.thumbnail.clone(),
                fallback: job.download_type.label(),
            }
            div { class: "library-info",
                strong { "{job.title}" }
                span { "{format_summary}" }
                small { "{job.output_hint}" }
                div { class: "card-actions library-actions",
                    button {
                        onclick: move |_| reveal_output(&output_hint),
                        "{i18n::t(\"reveal\")}"
                    }
                    button {
                        onclick: move |_| requeue_job(ctx, requeue.clone()),
                        "{i18n::t(\"re_download\")}"
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn ThumbnailBlock(class: String, src: String, fallback: String) -> Element {
    rsx! {
        div { class: "{class}",
            if src.trim().is_empty() {
                span { "{fallback}" }
            } else {
                img {
                    src: "{src}",
                    alt: "{fallback}",
                    loading: "lazy",
                }
            }
        }
    }
}

fn library_format_summary(job: &DownloadJob) -> String {
    match job.download_type {
        DownloadType::AudioOnly => format!(
            "{} / {} {}",
            job.download_type.label(),
            job.audio_format.to_uppercase(),
            job.audio_quality
        ),
        DownloadType::VideoOnly => format!(
            "{} / {} / {}",
            job.download_type.label(),
            job.video_codec,
            job.resolution_cap
        ),
        DownloadType::FullVideo => {
            format!(
                "{} / {} / {}",
                localized_format_label(&job.format_label),
                job.container.to_uppercase(),
                job.resolution_cap
            )
        }
    }
}
