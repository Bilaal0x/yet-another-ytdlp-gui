use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn HomeView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let input = (ctx.url_text)();
    let intake = parse_url_intake(&input);
    let busy = (ctx.busy)();
    let analysis_running = (ctx.analysis_running)();
    let can_analyze = intake.can_analyze() && !busy && !analysis_running;
    let url_count = i18n::t_with("url_count", &[("count", intake.urls.len().to_string())]);
    let invalid_count = i18n::t_with(
        "url_invalid_count",
        &[("count", intake.invalid_lines.len().to_string())],
    );
    let invalid_preview = intake
        .invalid_lines
        .iter()
        .take(4)
        .cloned()
        .collect::<Vec<_>>();

    rsx! {
        section { class: "screen-grid home-grid",
            div { class: "main-column",
                div { class: "paste-panel",
                    div { class: "panel-heading",
                        div { class: "eyebrow", "{i18n::t(\"home_start_eyebrow\")}" }
                        h2 { "{i18n::t(\"home_title\")}" }
                        p { "{i18n::t(\"home_intro\")}" }
                    }
                    textarea {
                        class: "url-box",
                        placeholder: "https://www.youtube.com/watch?v=...",
                        value: "{input}",
                        oninput: move |event| ctx.url_text.set(event.value()),
                    }
                    div { class: "url-feedback",
                        if intake.urls.is_empty() {
                            span { class: "status-pill muted", "{i18n::t(\"url_input_empty\")}" }
                        } else {
                            span { class: "summary-chip", "{url_count}" }
                        }
                        if !intake.invalid_lines.is_empty() {
                            span { class: "status-pill missing", "{invalid_count}" }
                        }
                        p { class: "muted-line", "{i18n::t(\"url_input_hint\")}" }
                    }
                    if !invalid_preview.is_empty() {
                        ul { class: "invalid-list",
                            for line in invalid_preview {
                                li { "{line}" }
                            }
                        }
                    }
                    div { class: "action-row",
                        button {
                            class: "primary-button",
                            disabled: !can_analyze,
                            onclick: move |_| start_analysis(ctx),
                            if analysis_running { "{i18n::t(\"analyzing\")}" } else { "{i18n::t(\"analyze_url\")}" }
                        }
                        if analysis_running {
                            button {
                                class: "secondary-button",
                                onclick: move |_| cancel_analysis(ctx),
                                "{i18n::t(\"cancel_analysis\")}"
                            }
                        } else {
                            button {
                                class: "secondary-button",
                                onclick: move |_| import_url_list(ctx),
                                "{i18n::t(\"import_list\")}"
                            }
                        }
                    }
                }
            }

            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"quick_preset\")}" }
                    h3 { "{i18n::t(\"production_profiles\")}" }
                }
                for (index, preset) in ctx.presets.read().iter().cloned().enumerate() {
                    PresetButton { index, preset }
                }
            }
        }
    }
}
