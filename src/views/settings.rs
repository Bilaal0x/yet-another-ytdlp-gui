use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn SettingsView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let deps = (ctx.dependencies)();
    let checking = (ctx.busy)();
    let settings = ctx.settings();
    let ready = deps.ytdlp.installed && deps.ffmpeg.installed;
    let language = settings.language.clone();
    let output_folder = settings.output_folder.clone();
    let settings_title = Screen::Settings.label();
    let settings_eyebrow = i18n::t("settings_eyebrow");
    let settings_intro = i18n::t("settings_intro");
    let dependencies_label = i18n::t("dependencies");
    let check_again_label = i18n::t("check_again");
    let checking_label = i18n::t("checking");
    let desktop_status_label = i18n::t("desktop_status");
    let ytdlp_ready_label = i18n::t("managed_ytdlp_ready");
    let ffmpeg_ready_label = i18n::t("managed_ffmpeg_ready");
    let action_label = if checking {
        checking_label.clone()
    } else {
        check_again_label
    };
    let status_label = if ready {
        Screen::Ready.label()
    } else if checking {
        checking_label
    } else {
        i18n::t("needs_setup")
    };

    rsx! {
        section { class: "screen-grid",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{settings_eyebrow}" }
                    h2 { "{settings_title}" }
                    p { "{settings_intro}" }
                }
                div { class: "settings-sections",
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"general\")}" }
                        label { class: "field-label",
                            span { "{i18n::t(\"language\")}" }
                            LanguageSelector { current_language: language }
                        }
                        label { class: "field-label",
                            span { "{i18n::t(\"default_download_folder\")}" }
                            input {
                                value: "{output_folder}",
                                oninput: move |event| {
                                    ctx.settings.with_mut(|settings| settings.output_folder = event.value());
                                },
                            }
                        }
                        div { class: "settings-actions",
                            button {
                                class: "secondary-button",
                                onclick: move |_| pick_output_folder(ctx),
                                "{i18n::t(\"browse_folder\")}"
                            }
                        }
                    }
                    div { class: "settings-section dependency-cards",
                        h3 { "{dependencies_label}" }
                        DependencyCard { item: deps.ytdlp.clone() }
                        DependencyCard { item: deps.ffmpeg.clone() }
                        div { class: "settings-actions",
                            button {
                                class: "secondary-button",
                                disabled: checking,
                                onclick: move |_| ctx.backend.send(BackendAction::RefreshDependencies),
                                "{action_label}"
                            }
                        }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"downloads\")}" }
                        RangeSetting {
                            label: i18n::t("max_parallel_jobs"),
                            field: "parallel_jobs".to_string(),
                            value: settings.parallel_jobs,
                            min: 1,
                            max: 5,
                        }
                        RangeSetting {
                            label: i18n::t("concurrent_fragments"),
                            field: "concurrent_fragments".to_string(),
                            value: settings.concurrent_fragments,
                            min: 1,
                            max: 12,
                        }
                        RangeSetting {
                            label: i18n::t("retries"),
                            field: "retries".to_string(),
                            value: settings.retries,
                            min: 0,
                            max: 10,
                        }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"privacy\")}" }
                        div { class: "settings-actions",
                            button {
                                class: "secondary-button",
                                onclick: move |_| clear_history(ctx),
                                "{i18n::t(\"clear_local_history\")}"
                            }
                            button {
                                class: "ghost-button",
                                onclick: move |_| forget_cookie_file(ctx),
                                "{i18n::t(\"forget_cookie_file\")}"
                            }
                        }
                    }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{desktop_status_label}" }
                    h3 { "{status_label}" }
                }
                CheckLine { text: ytdlp_ready_label, ok: deps.ytdlp.installed }
                CheckLine { text: ffmpeg_ready_label, ok: deps.ffmpeg.installed }
                CheckLine { text: i18n::t("output_folder_configured"), ok: !settings.output_folder.trim().is_empty() }
            }
        }
    }
}
