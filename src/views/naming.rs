use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn NamingView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let settings = ctx.settings();
    let output_folder = settings.output_folder.clone();
    let file_template = settings.file_template.clone();
    let variables = [
        "title",
        "uploader",
        "upload_date",
        "playlist",
        "playlist_index",
        "ext",
    ];
    let preview = preview_output_path(&settings);
    let template_valid = template_is_valid(&file_template);
    let command = queue_command_preview(ctx);

    rsx! {
        section { class: "screen-grid",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"naming_output\")}" }
                    h2 { "{Screen::Naming.label()}" }
                    p { "{i18n::t(\"naming_intro\")}" }
                }
                div { class: "field-stack",
                    label { class: "field-label",
                        span { "{i18n::t(\"folder\")}" }
                        div { class: "field-with-button",
                            input {
                                value: "{output_folder}",
                                oninput: move |event| {
                                    ctx.settings.with_mut(|settings| settings.output_folder = event.value());
                                },
                            }
                            button {
                                class: "secondary-button",
                                onclick: move |_| pick_output_folder(ctx),
                                "{i18n::t(\"browse\")}"
                            }
                        }
                    }
                    label { class: "field-label",
                        span { "{i18n::t(\"filename_template\")}" }
                        input {
                            value: "{file_template}",
                            oninput: move |event| {
                                ctx.settings.with_mut(|settings| settings.file_template = event.value());
                            },
                        }
                    }
                }
                div { class: "helper-chip-row",
                    for variable in variables {
                        button {
                            class: "helper-chip",
                            onclick: move |_| {
                                ctx.settings.with_mut(|settings| {
                                    settings.file_template.push_str(&format!("%({variable})s"));
                                });
                            },
                            "{variable}"
                        }
                    }
                }
                div { class: "preview-paths",
                    div { class: "eyebrow", "{i18n::t(\"live_preview\")}" }
                    code { "{preview}" }
                }
                div { class: "option-grid two-col",
                    ToggleSetting {
                        label: i18n::t("create_playlist_folders"),
                        field: "create_playlist_folders".to_string(),
                        value: settings.create_playlist_folders,
                    }
                    ToggleSetting {
                        label: i18n::t("add_counter"),
                        field: "add_playlist_index".to_string(),
                        value: settings.add_playlist_index,
                    }
                    ToggleSetting {
                        label: i18n::t("replace_unsafe_characters"),
                        field: "replace_unsafe_characters".to_string(),
                        value: settings.replace_unsafe_characters,
                    }
                    ToggleSetting {
                        label: i18n::t("prevent_overwrites"),
                        field: "prevent_overwrites".to_string(),
                        value: settings.prevent_overwrites,
                    }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"template_health\")}" }
                    h3 {
                        if template_valid {
                            "{i18n::t(\"looks_valid\")}"
                        } else {
                            "{i18n::t(\"template_needs_title_ext\")}"
                        }
                    }
                }
                CheckLine { text: i18n::t("check_output_folder_stored"), ok: !output_folder.trim().is_empty() }
                CheckLine { text: i18n::t("check_template_includes_title"), ok: file_template.contains("%(title)s") }
                CheckLine { text: i18n::t("check_template_preserves_extension"), ok: file_template.contains("%(ext)s") }
                code { class: "command-code block", "{command}" }
                button {
                    class: "primary-button full",
                    onclick: move |_| open_ready_view(ctx),
                    "{i18n::t(\"apply_naming\")}"
                }
            }
        }
    }
}
