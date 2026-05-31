use super::super::*;
use super::LanguageSelector;

#[component]
pub(crate) fn Sidebar() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let nav_items = [
        Screen::Home,
        Screen::Queue,
        Screen::Library,
        Screen::Presets,
        Screen::Settings,
    ];

    rsx! {
        aside { class: "sidebar",
            div { class: "brand-block",
                div { class: "brand-mark", "Y" }
                div {
                    div { class: "brand-name", "{i18n::t(\"app_name\")}" }
                    div { class: "brand-subtitle", "{i18n::t(\"app_subtitle\")}" }
                }
            }

            nav { class: "primary-nav",
                for target in nav_items {
                    button {
                        class: if (ctx.screen)() == target { "nav-item active" } else { "nav-item" },
                        onclick: move |_| ctx.screen.set(target),
                        span { class: "nav-label", "{target.label()}" }
                        span { class: "nav-caption", "{target.caption()}" }
                    }
                }
            }

            div { class: "sidebar-panel",
                div { class: "eyebrow", "{i18n::t(\"active_preset\")}" }
                div { class: "profile-name", "{ctx.active_preset().name}" }
                p { "{i18n::t(\"active_preset_help\")}" }
                button {
                    class: "text-button",
                    onclick: move |_| ctx.screen.set(Screen::Advanced),
                    "{i18n::t(\"review_command\")}"
                }
            }
        }
    }
}

#[component]
pub(crate) fn TopBar() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let language = ctx.language();
    let title = (ctx.screen)().label();
    let crumb = i18n::t_with("desktop_crumb", &[("title", title.clone())]);

    rsx! {
        header { class: "topbar",
            div {
                div { class: "crumb", "{crumb}" }
                h1 { "{title}" }
            }
            div { class: "topbar-actions",
                LanguageSelector { current_language: language }
                button {
                    class: "ghost-button",
                    onclick: move |_| ctx.screen.set(Screen::Naming),
                    "{Screen::Naming.label()}"
                }
                button {
                    class: "ghost-button",
                    onclick: move |_| ctx.screen.set(Screen::Error),
                    "{Screen::Error.label()}"
                }
            }
        }
    }
}
