use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn ReadyView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let analysis = (ctx.analysis)();
    let show_cli = (ctx.show_cli)();

    match analysis {
        Some(result) => {
            let first = result.items.first().cloned();
            let item_count = result.items.len();
            let item_count_label = i18n::t_with("item_count", &[("count", item_count.to_string())]);
            rsx! {
                section { class: "screen-grid inspector-layout",
                    div { class: "main-column",
                        div { class: "ready-banner",
                            div {
                                div { class: "eyebrow", "{i18n::t(\"ready_analysis_complete\")}" }
                                h2 { "{i18n::t(\"ready_to_queue\")}" }
                            }
                            div { class: "summary-chip warm", "{item_count_label}" }
                        }

                        if let Some(item) = first {
                            MediaPreview { item }
                        }

                        div { class: "choice-row three-up",
                            ModeButton { mode: DownloadType::FullVideo, label: DownloadType::FullVideo.label(), detail: i18n::t("mode_full_video_detail") }
                            ModeButton { mode: DownloadType::AudioOnly, label: i18n::t("mode_extract_audio"), detail: i18n::t("mode_extract_audio_detail") }
                            ModeButton { mode: DownloadType::VideoOnly, label: DownloadType::VideoOnly.label(), detail: i18n::t("mode_video_only_detail") }
                        }

                        div { class: "command-strip",
                            button {
                                class: "text-button",
                                onclick: move |_| ctx.show_cli.set(!show_cli),
                                if show_cli { "{i18n::t(\"hide_cli\")}" } else { "{i18n::t(\"show_cli\")}" }
                            }
                            if show_cli {
                                code { class: "command-code", "{queue_command_preview(ctx)}" }
                            } else {
                                span { "{i18n::t(\"command_preview_collapsed\")}" }
                            }
                        }
                    }

                    aside { class: "side-panel inspector-panel",
                        InspectorSettings {}
                        button {
                            class: "primary-button full",
                            onclick: move |_| add_analysis_to_queue(ctx),
                            "{i18n::t(\"add_to_queue\")}"
                        }
                        button {
                            class: "secondary-button full",
                            onclick: move |_| ctx.screen.set(Screen::Format),
                            "{i18n::t(\"customize_format\")}"
                        }
                        button {
                            class: "ghost-button full",
                            onclick: move |_| ctx.screen.set(Screen::Playlist),
                            "{i18n::t(\"review_items\")}"
                        }
                    }
                }
            }
        }
        None => {
            rsx! { EmptyState { title: i18n::t("empty_nothing_analyzed_title"), message: i18n::t("empty_nothing_analyzed_message") } }
        }
    }
}
