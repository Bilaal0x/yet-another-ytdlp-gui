use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn ErrorView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let show_debug = (ctx.show_debug)();
    let error = (ctx.last_error)().unwrap_or_else(|| {
        AppError::new(i18n::t("no_active_issue"), i18n::t("diagnostics_clear"), "")
    });

    rsx! {
        section { class: "screen-grid inspector-layout",
            div { class: "main-column",
                div { class: "attention-card",
                    div { class: "warning-mark", "!" }
                    div {
                        div { class: "eyebrow", "{i18n::t(\"diagnostics\")}" }
                        h2 { "{error.title}" }
                        p { "{error.message}" }
                    }
                    div { class: "action-row",
                        button { class: "primary-button", onclick: move |_| ctx.backend.send(BackendAction::RetryFailed { settings: ctx.settings() }), "{i18n::t(\"fix_and_retry\")}" }
                        button { class: "secondary-button", onclick: move |_| ctx.backend.send(BackendAction::RefreshDependencies), "{i18n::t(\"recheck_dependencies\")}" }
                        button { class: "ghost-button", onclick: move |_| ctx.screen.set(Screen::Settings), "{i18n::t(\"open_settings\")}" }
                    }
                }
                div { class: "preview-card affected-download",
                    div { class: "thumbnail-block small", "{i18n::t(\"hint\")}" }
                    div {
                        strong { "{error.suggestion}" }
                        span { "{i18n::t(\"failure_remediation\")}" }
                    }
                }
                div { class: "debug-panel",
                    button { class: "text-button", onclick: move |_| ctx.show_debug.set(!show_debug), if show_debug { "{i18n::t(\"hide_debug_output\")}" } else { "{i18n::t(\"show_debug_output\")}" } }
                    if show_debug {
                        pre { "{error.debug}" }
                    }
                }
            }
            aside { class: "side-panel inspector-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"common_causes\")}" }
                    h3 { "{i18n::t(\"troubleshooting\")}" }
                }
                TroubleLine { title: i18n::t("yt_dlp_missing"), detail: i18n::t("check_ytdlp_missing") }
                TroubleLine { title: i18n::t("ffmpeg_missing"), detail: i18n::t("check_ffmpeg_missing") }
                TroubleLine { title: i18n::t("cookies_needed"), detail: i18n::t("check_cookies_needed") }
                TroubleLine { title: i18n::t("format_unavailable"), detail: i18n::t("check_format_unavailable") }
            }
        }
    }
}
