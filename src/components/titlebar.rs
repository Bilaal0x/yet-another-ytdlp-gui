use super::super::*;

#[cfg(feature = "desktop")]
#[component]
pub(crate) fn AppTitleBar() -> Element {
    let window = dioxus::desktop::use_window();
    let drag_window = window.clone();
    let maximize_drag_window = window.clone();
    let minimize_window = window.clone();
    let maximize_window = window.clone();
    let close_window = window.clone();

    rsx! {
        header { class: "custom-titlebar",
            div { class: "titlebar-brand",
                div { class: "titlebar-mark", "Y" }
            }
            div {
                class: "titlebar-drag-region",
                onmousedown: move |_| drag_window.drag(),
                ondoubleclick: move |_| maximize_drag_window.toggle_maximized(),
                span { class: "titlebar-title", "yaydlp" }
                span { class: "titlebar-subtitle", "Yet Another YT-DLP GUI" }
            }
            div { class: "window-controls",
                button {
                    class: "window-button",
                    title: "Minimize",
                    onclick: move |_| minimize_window.set_minimized(true),
                    span { class: "window-glyph minimize" }
                }
                button {
                    class: "window-button",
                    title: "Maximize",
                    onclick: move |_| maximize_window.toggle_maximized(),
                    span { class: "window-glyph maximize" }
                }
                button {
                    class: "window-button close",
                    title: "Close",
                    onclick: move |_| close_window.close(),
                    span { class: "window-glyph close" }
                }
            }
        }
    }
}

#[cfg(not(feature = "desktop"))]
#[component]
pub(crate) fn AppTitleBar() -> Element {
    rsx! {
        header { class: "custom-titlebar",
            div { class: "titlebar-brand",
                div { class: "titlebar-mark", "Y" }
            }
            div { class: "titlebar-drag-region",
                span { class: "titlebar-title", "yaydlp" }
                span { class: "titlebar-subtitle", "Yet Another YT-DLP GUI" }
            }
        }
    }
}
