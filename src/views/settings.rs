use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn SettingsView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let settings = ctx.settings();
    let _language = ctx.language();
    let deps = (ctx.dependencies)();
    let ready = deps.ytdlp.installed && deps.ffmpeg.installed;
    let ready_label = Screen::Ready.label();
    let status_label = if ready {
        ready_label
    } else {
        i18n::t("needs_setup")
    };

    rsx! {
        section { class: "screen-grid inspector-layout",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"settings_eyebrow\")}" }
                    h2 { "{Screen::Settings.label()}" }
                    p { "{i18n::t(\"settings_intro\")}" }
                }
                div { class: "settings-sections",
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"general\")}" }
                        label {
                            class: "field-label",
                            span { "{i18n::t(\"language\")}" }
                            LanguageSelector { current_language: settings.language.clone() }
                        }
                        label { class: "field-label", span { "{i18n::t(\"default_download_folder\")}" } input { value: "{settings.output_folder}", oninput: move |event| ctx.settings.with_mut(|s| s.output_folder = event.value()) } }
                        div { class: "settings-actions",
                            button { class: "secondary-button", onclick: move |_| pick_output_folder(ctx), "{i18n::t(\"browse_folder\")}" }
                        }
                    }
                    div { class: "settings-section dependency-cards",
                        h3 { "{i18n::t(\"dependencies\")}" }
                        DependencyCard { item: deps.ytdlp.clone() }
                        DependencyCard { item: deps.ffmpeg.clone() }
                        div { class: "settings-actions",
                            button { class: "secondary-button", onclick: move |_| ctx.backend.send(BackendAction::RefreshDependencies), "{i18n::t(\"check_again\")}" }
                        }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"downloads\")}" }
                        RangeSetting { label: i18n::t("max_parallel_jobs"), field: "parallel_jobs".to_string(), value: settings.parallel_jobs, min: 1, max: 5 }
                        RangeSetting { label: i18n::t("concurrent_fragments"), field: "concurrent_fragments".to_string(), value: settings.concurrent_fragments, min: 1, max: 12 }
                        RangeSetting { label: i18n::t("retries"), field: "retries".to_string(), value: settings.retries, min: 0, max: 10 }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"privacy\")}" }
                        div { class: "settings-actions",
                            button { class: "secondary-button", onclick: move |_| clear_history(ctx), "{i18n::t(\"clear_local_history\")}" }
                            button { class: "ghost-button", onclick: move |_| ctx.settings.with_mut(|s| s.cookie_file.clear()), "{i18n::t(\"forget_cookie_file\")}" }
                        }
                    }
                }
            }
            aside { class: "side-panel inspector-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"desktop_status\")}" }
                    h3 { "{status_label}" }
                }
                CheckLine { text: i18n::t("managed_ytdlp_ready"), ok: deps.ytdlp.installed }
                CheckLine { text: i18n::t("managed_ffmpeg_ready"), ok: deps.ffmpeg.installed }
                CheckLine { text: i18n::t("output_folder_configured"), ok: !settings.output_folder.trim().is_empty() }
            }
        }
    }
}
