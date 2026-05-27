use super::super::*;

#[component]
pub(crate) fn Sidebar() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let nav_items = vec![
        (
            Screen::Home,
            Screen::Home.label(),
            i18n::t("nav_new_download_caption"),
        ),
        (
            Screen::Queue,
            Screen::Queue.label(),
            i18n::t("nav_queue_caption"),
        ),
        (
            Screen::Library,
            Screen::Library.label(),
            i18n::t("nav_library_caption"),
        ),
        (
            Screen::Presets,
            Screen::Presets.label(),
            i18n::t("nav_presets_caption"),
        ),
        (
            Screen::Settings,
            Screen::Settings.label(),
            i18n::t("nav_settings_caption"),
        ),
    ];
    let preset = ctx.active_preset();
    let app_name = i18n::t("app_name");
    let app_subtitle = i18n::t("app_subtitle");
    let active_preset = i18n::t("active_preset");
    let active_preset_help = i18n::t("active_preset_help");
    let review_command = i18n::t("review_command");

    rsx! {
        aside { class: "sidebar",
            div { class: "brand-block",
                div { class: "brand-mark", "F" }
                div {
                    div { class: "brand-name", "{app_name}" }
                    div { class: "brand-subtitle", "{app_subtitle}" }
                }
            }

            nav { class: "primary-nav",
                for (target, label, caption) in nav_items {
                    button {
                        class: if (ctx.screen)() == target { "nav-item active" } else { "nav-item" },
                        onclick: move |_| ctx.screen.set(target),
                        span { class: "nav-label", "{label}" }
                        span { class: "nav-caption", "{caption}" }
                    }
                }
            }

            div { class: "sidebar-panel",
                div { class: "eyebrow", "{active_preset}" }
                div { class: "profile-name", "{preset.name}" }
                p { "{active_preset_help}" }
                button {
                    class: "text-button",
                    onclick: move |_| ctx.screen.set(Screen::Advanced),
                    "{review_command}"
                }
            }
        }
    }
}
#[component]
pub(crate) fn TopBar() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let title = (ctx.screen)().label();
    let crumb = i18n::t_with("desktop_crumb", &[("title", title.clone())]);
    let url_count = parse_urls(&(ctx.url_text)()).len();
    let url_summary = i18n::t_with("url_count", &[("count", url_count.to_string())]);
    let running = ctx
        .jobs
        .read()
        .iter()
        .filter(|job| job.status == JobStatus::Running)
        .count();
    let running_summary = i18n::t_with("running_count", &[("count", running.to_string())]);
    let analysis_running = (ctx.analysis_running)();
    let save_location = Screen::Naming.label();
    let diagnostics = i18n::t("diagnostics");

    rsx! {
        header { class: "topbar",
            div {
                div { class: "crumb", "{crumb}" }
                h1 { "{title}" }
            }
            div { class: "topbar-actions",
                div { class: "summary-chip", "{url_summary}" }
                if running > 0 {
                    div { class: "summary-chip warm", "{running_summary}" }
                }
                if analysis_running {
                    button {
                        class: "secondary-button",
                        onclick: move |_| cancel_analysis(ctx),
                        "{i18n::t(\"cancel_analysis\")}"
                    }
                }
                button {
                    class: "ghost-button",
                    onclick: move |_| ctx.screen.set(Screen::Naming),
                    "{save_location}"
                }
                button {
                    class: "ghost-button",
                    onclick: move |_| ctx.screen.set(Screen::Error),
                    "{diagnostics}"
                }
            }
        }
    }
}
