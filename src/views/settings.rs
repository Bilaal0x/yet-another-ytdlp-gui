use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn SettingsView() -> Element {
    let ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let deps = (ctx.dependencies)();
    let checking = (ctx.busy)();
    let ready = deps.ytdlp.installed && deps.ffmpeg.installed;
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
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{desktop_status_label}" }
                    h3 { "{status_label}" }
                }
                CheckLine { text: ytdlp_ready_label, ok: deps.ytdlp.installed }
                CheckLine { text: ffmpeg_ready_label, ok: deps.ffmpeg.installed }
            }
        }
    }
}
