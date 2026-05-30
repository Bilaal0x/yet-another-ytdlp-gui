use super::super::*;

#[component]
pub(crate) fn Sidebar() -> Element {
    let mut ctx = use_context::<FetchContext>();
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
                    div { class: "brand-name", "yaydlp" }
                    div { class: "brand-subtitle", "Yet Another YT-DLP GUI" }
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
                div { class: "eyebrow", "Active preset" }
                div { class: "profile-name", "{ctx.active_preset().name}" }
                p { "Every queue item will keep its own command, output path, status, and diagnostic log." }
                button {
                    class: "text-button",
                    onclick: move |_| ctx.screen.set(Screen::Advanced),
                    "Review command"
                }
            }
        }
    }
}

#[component]
pub(crate) fn TopBar() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let title = (ctx.screen)().label();
    let crumb = format!("yaydlp Desktop / {title}");

    rsx! {
        header { class: "topbar",
            div {
                div { class: "crumb", "{crumb}" }
                h1 { "{title}" }
            }
            div { class: "topbar-actions",
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
