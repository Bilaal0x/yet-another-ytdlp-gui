use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn HomeView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let busy = (ctx.busy)();
    let analysis_running = (ctx.analysis_running)();
    let urls = parse_urls(&(ctx.url_text)());
    let can_analyze = !urls.is_empty() && !busy;

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
                        value: "{ctx.url_text}",
                        oninput: move |event| ctx.url_text.set(event.value()),
                    }
                    div { class: "action-row",
                        button {
                            class: "primary-button",
                            disabled: !can_analyze,
                            onclick: move |_| {
                                let urls = parse_urls(&(ctx.url_text)());
                                if urls.is_empty() {
                                    ctx.last_error.set(Some(AppError::new(
                                        i18n::t("error_no_url_title"),
                                        i18n::t("error_no_url_message"),
                                        "",
                                    )));
                                    ctx.screen.set(Screen::Error);
                                    return;
                                }
                                ctx.backend.send(BackendAction::Analyze {
                                    urls,
                                    settings: ctx.settings(),
                                });
                            },
                            if busy { "{i18n::t(\"analyzing\")}" } else { "{i18n::t(\"analyze_url\")}" }
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
