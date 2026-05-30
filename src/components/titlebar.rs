use super::super::*;

#[cfg(feature = "desktop")]
#[component]
pub(crate) fn AppTitleBar() -> Element {
    let ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let app_name = i18n::t("app_name");
    let app_subtitle = i18n::t("app_subtitle");
    let minimize = i18n::t("minimize");
    let maximize = i18n::t("maximize");
    let close = i18n::t("close");
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
                span { class: "titlebar-title", "{app_name}" }
                span { class: "titlebar-subtitle", "{app_subtitle}" }
            }
            div { class: "window-controls",
                button {
                    class: "window-button",
                    title: "{minimize}",
                    onclick: move |_| minimize_window.set_minimized(true),
                    span { class: "window-glyph minimize" }
                }
                button {
                    class: "window-button",
                    title: "{maximize}",
                    onclick: move |_| maximize_window.toggle_maximized(),
                    span { class: "window-glyph maximize" }
                }
                button {
                    class: "window-button close",
                    title: "{close}",
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
    let ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let app_name = i18n::t("app_name");
    let app_subtitle = i18n::t("app_subtitle");

    rsx! {
        header { class: "custom-titlebar",
            div { class: "titlebar-brand",
                div { class: "titlebar-mark", "Y" }
            }
            div { class: "titlebar-drag-region",
                span { class: "titlebar-title", "{app_name}" }
                span { class: "titlebar-subtitle", "{app_subtitle}" }
            }
        }
    }
}
