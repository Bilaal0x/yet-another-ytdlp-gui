use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn AdvancedView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let settings = ctx.settings();
    let subtitle_languages = settings.subtitle_languages.clone();
    let proxy = settings.proxy.clone();
    let cookie_file = settings.cookie_file.clone();
    let command = queue_command_preview(ctx);

    rsx! {
        section { class: "screen-grid",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"advanced_power_controls\")}" }
                    h2 { "{Screen::Advanced.label()}" }
                    p { "{i18n::t(\"advanced_intro\")}" }
                }
                div { class: "settings-sections",
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"subtitles\")}" }
                        ToggleSetting {
                            label: i18n::t("download_subtitles"),
                            field: "write_subtitles".to_string(),
                            value: settings.write_subtitles,
                        }
                        ToggleSetting {
                            label: i18n::t("auto_subtitles"),
                            field: "write_auto_subtitles".to_string(),
                            value: settings.write_auto_subtitles,
                        }
                        label { class: "field-label",
                            span { "{i18n::t(\"languages\")}" }
                            input {
                                value: "{subtitle_languages}",
                                oninput: move |event| {
                                    ctx.settings.with_mut(|settings| settings.subtitle_languages = event.value());
                                },
                            }
                        }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"metadata\")}" }
                        ToggleSetting {
                            label: i18n::t("write_thumbnail"),
                            field: "write_thumbnail".to_string(),
                            value: settings.write_thumbnail,
                        }
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
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"network\")}" }
                        label { class: "field-label",
                            span { "Proxy" }
                            input {
                                placeholder: "http://127.0.0.1:8080",
                                value: "{proxy}",
                                oninput: move |event| {
                                    ctx.settings.with_mut(|settings| settings.proxy = event.value());
                                },
                            }
                        }
                        RangeSetting {
                            label: i18n::t("retries"),
                            field: "retries".to_string(),
                            value: settings.retries,
                            min: 0,
                            max: 10,
                        }
                        SelectField {
                            label: i18n::t("rate_limit"),
                            value: settings.speed_limit.clone(),
                            options: vec![
                                "Unlimited".to_string(),
                                "10M".to_string(),
                                "5M".to_string(),
                                "1M".to_string(),
                            ],
                            target: "speed".to_string(),
                        }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"authentication\")}" }
                        label { class: "field-label",
                            span { "{i18n::t(\"cookie_file\")}" }
                            input {
                                value: "{cookie_file}",
                                oninput: move |event| {
                                    ctx.settings.with_mut(|settings| settings.cookie_file = event.value());
                                },
                            }
                        }
                        button {
                            class: "secondary-button",
                            onclick: move |_| pick_cookie_file(ctx),
                            "{i18n::t(\"choose_cookie_file\")}"
                        }
                    }
                    div { class: "settings-section",
                        h3 { "{i18n::t(\"playlist_behavior\")}" }
                        ToggleSetting {
                            label: i18n::t("skip_existing"),
                            field: "skip_existing".to_string(),
                            value: settings.skip_existing,
                        }
                        ToggleSetting {
                            label: i18n::t("no_overwrites"),
                            field: "prevent_overwrites".to_string(),
                            value: settings.prevent_overwrites,
                        }
                    }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"command_preview\")}" }
                    h3 { "{i18n::t(\"next_job\")}" }
                }
                code { class: "command-code block", "{command}" }
                button {
                    class: "primary-button full",
                    onclick: move |_| open_ready_view(ctx),
                    "{i18n::t(\"apply_options\")}"
                }
            }
        }
    }
}
