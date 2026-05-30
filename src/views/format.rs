use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn FormatView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let presets = [
        (
            "Best quality",
            "bestvideo+bestaudio/best",
            "Source",
            "format_largest",
            "MKV",
            "Best",
        ),
        (
            "MP4 1080p",
            "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]",
            "1080p",
            "format_balanced",
            "MP4",
            "H.264",
        ),
        (
            "MP4 720p",
            "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720]",
            "720p",
            "format_smaller",
            "MP4",
            "H.264",
        ),
        (
            "4K if available",
            "bestvideo[height<=2160]+bestaudio/best[height<=2160]",
            "2160p",
            "format_large",
            "MKV",
            "Best",
        ),
        (
            "Video only",
            "bestvideo",
            "Source",
            "format_no_audio",
            "MKV",
            "Best",
        ),
    ];
    let current_format = (ctx.selected_format)();
    let current_rule = format_rule(
        &current_format,
        &(ctx.container)(),
        &(ctx.video_codec)(),
        &(ctx.resolution_cap)(),
    );
    let command = queue_command_preview(ctx);

    rsx! {
        section { class: "screen-grid",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"format_eyebrow\")}" }
                    h2 { "{Screen::Format.label()}" }
                    p { "{i18n::t(\"format_intro\")}" }
                }
                div { class: "preset-card-grid",
                    for (label, rule, cap, estimate_key, container, codec) in presets {
                        button {
                            class: if current_format == label { "format-card active" } else { "format-card" },
                            onclick: move |_| {
                                ctx.selected_format.set(label.to_string());
                                ctx.container.set(container.to_string());
                                ctx.video_codec.set(codec.to_string());
                                ctx.resolution_cap.set(cap.to_string());
                                ctx.download_type.set(if label == "Video only" {
                                    DownloadType::VideoOnly
                                } else {
                                    DownloadType::FullVideo
                                });
                            },
                            span { class: "format-title", "{localized_format_label(label)}" }
                            span { "{i18n::t(\"resolution\")}: {localized_format_cap(cap)}" }
                            span { "{i18n::t(\"profile\")}: {i18n::t(estimate_key)}" }
                            code { "{rule}" }
                        }
                    }
                }
                div { class: "advanced-row",
                    SelectField {
                        label: i18n::t("container"),
                        value: (ctx.container)(),
                        options: vec!["MP4".to_string(), "MKV".to_string(), "WebM".to_string()],
                        target: "container".to_string(),
                    }
                    SelectField {
                        label: i18n::t("video_codec"),
                        value: (ctx.video_codec)(),
                        options: vec!["H.264".to_string(), "AV1".to_string(), "Best".to_string()],
                        target: "codec".to_string(),
                    }
                    SelectField {
                        label: i18n::t("resolution_cap"),
                        value: (ctx.resolution_cap)(),
                        options: vec![
                            "720p".to_string(),
                            "1080p".to_string(),
                            "1440p".to_string(),
                            "2160p".to_string(),
                            "Source".to_string(),
                        ],
                        target: "resolution".to_string(),
                    }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"generated_argument\")}" }
                    h3 { "{i18n::t(\"format_rule_label\")}" }
                }
                code { class: "code-pill", "{current_rule}" }
                code { class: "command-code block", "{command}" }
                button {
                    class: "primary-button full",
                    onclick: move |_| open_ready_view(ctx),
                    "{i18n::t(\"save_choice\")}"
                }
            }
        }
    }
}
