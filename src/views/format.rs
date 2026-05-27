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
            "No cap",
            "format_largest",
        ),
        (
            "MP4 1080p",
            "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]",
            "1080p",
            "format_balanced",
        ),
        (
            "MP4 720p",
            "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720]",
            "720p",
            "format_smaller",
        ),
        (
            "4K if available",
            "bestvideo[height<=2160]+bestaudio/best[height<=2160]",
            "2160p",
            "format_large",
        ),
        ("Video only", "bestvideo", "Source", "format_no_audio"),
    ];

    rsx! {
        section { class: "screen-grid inspector-layout",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"format_eyebrow\")}" }
                    h2 { "{Screen::Format.label()}" }
                    p { "{i18n::t(\"format_intro\")}" }
                }
                div { class: "preset-card-grid",
                    for (label, rule, cap, estimate_key) in presets {
                        {
                            let format_title = localized_format_label(label);
                            let cap_label = localized_format_cap(cap);
                            let estimate = i18n::t(estimate_key);
                            rsx! {
                        button {
                            class: if (ctx.selected_format)() == label { "format-card active" } else { "format-card" },
                            onclick: move |_| {
                                ctx.selected_format.set(label.to_string());
                                if label.contains("720") { ctx.resolution_cap.set("720p".to_string()); }
                                if label.contains("1080") { ctx.resolution_cap.set("1080p".to_string()); }
                                if label.contains("4K") { ctx.resolution_cap.set("4K".to_string()); }
                            },
                            span { class: "format-title", "{format_title}" }
                            span { "{i18n::t(\"resolution\")}: {cap_label}" }
                            span { "{i18n::t(\"profile\")}: {estimate}" }
                            code { "{rule}" }
                        }
                            }
                        }
                    }
                }
                div { class: "advanced-row",
                    SelectField { label: i18n::t("container"), value: (ctx.container)(), options: vec!["MP4".to_string(), "MKV".to_string(), "WebM".to_string()], target: "container".to_string() }
                    SelectField { label: i18n::t("video_codec"), value: (ctx.video_codec)(), options: vec!["H.264".to_string(), "AV1".to_string(), "Best".to_string()], target: "codec".to_string() }
                    SelectField { label: i18n::t("resolution_cap"), value: (ctx.resolution_cap)(), options: vec!["720p".to_string(), "1080p".to_string(), "4K".to_string()], target: "resolution".to_string() }
                }
            }
            aside { class: "side-panel inspector-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"generated_argument\")}" }
                    h3 { "{i18n::t(\"format_rule_label\")}" }
                }
                code { class: "code-pill", "{format_rule(&(ctx.selected_format)(), &(ctx.container)(), &(ctx.video_codec)(), &(ctx.resolution_cap)())}" }
                button { class: "primary-button full", onclick: move |_| open_ready_view(ctx), "{i18n::t(\"save_choice\")}" }
            }
        }
    }
}
