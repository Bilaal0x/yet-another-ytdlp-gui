use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn AudioView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let settings = ctx.settings();
    let formats = ["MP3", "FLAC", "M4A", "Opus", "WAV"];
    let selected_format = (ctx.selected_audio_format)();
    let audio_ext = selected_format.to_lowercase();
    let audio_preview = i18n::t_with("audio_preview_name", &[("ext", audio_ext)]);
    let command = queue_command_preview(ctx);

    rsx! {
        section { class: "screen-grid warm-workflow",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"music_workflow\")}" }
                    h2 { "{Screen::Audio.label()}" }
                    p { "{i18n::t(\"audio_intro\")}" }
                }
                div { class: "audio-format-grid",
                    for fmt in formats {
                        button {
                            class: if selected_format == fmt { "audio-card active" } else { "audio-card" },
                            onclick: move |_| {
                                ctx.download_type.set(DownloadType::AudioOnly);
                                ctx.selected_audio_format.set(fmt.to_string());
                            },
                            span { class: "audio-name", "{fmt}" }
                            span { "{audio_format_detail(fmt)}" }
                        }
                    }
                }
                div { class: "quality-bar",
                    SelectField {
                        label: i18n::t("quality"),
                        value: (ctx.audio_quality)(),
                        options: vec![
                            "Best".to_string(),
                            "320 kbps".to_string(),
                            "256 kbps".to_string(),
                            "192 kbps".to_string(),
                        ],
                        target: "audio_quality".to_string(),
                    }
                }
                div { class: "option-grid two-col",
                    ToggleSetting {
                        label: i18n::t("toggle_embed_thumbnail"),
                        field: "embed_thumbnail".to_string(),
                        value: settings.embed_thumbnail,
                    }
                    ToggleSetting {
                        label: i18n::t("toggle_add_metadata"),
                        field: "add_metadata".to_string(),
                        value: settings.add_metadata,
                    }
                    ToggleSetting {
                        label: i18n::t("toggle_split_playlist_into_tracks"),
                        field: "split_chapters".to_string(),
                        value: settings.split_chapters,
                    }
                    ToggleSetting {
                        label: i18n::t("toggle_keep_original_file"),
                        field: "keep_original".to_string(),
                        value: settings.keep_original,
                    }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"output_preview\")}" }
                    h3 { "{audio_preview}" }
                }
                DependencyLine { item: (ctx.dependencies)().ffmpeg }
                code { class: "command-code block", "{command}" }
                button {
                    class: "primary-button full",
                    onclick: move |_| add_analysis_to_queue(ctx),
                    "{i18n::t(\"add_audio_job\")}"
                }
                button {
                    class: "secondary-button full",
                    onclick: move |_| open_ready_view(ctx),
                    "{i18n::t(\"back_to_analysis\")}"
                }
            }
        }
    }
}
