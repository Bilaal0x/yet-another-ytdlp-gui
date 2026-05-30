use super::*;

#[component]
pub(crate) fn ActiveView() -> Element {
    let ctx = use_context::<FetchContext>();

    match (ctx.screen)() {
        Screen::Home => rsx! { ScreenPanel { screen: Screen::Home } },
        Screen::Ready => rsx! { ScreenPanel { screen: Screen::Ready } },
        Screen::Format => rsx! { ScreenPanel { screen: Screen::Format } },
        Screen::Audio => rsx! { ScreenPanel { screen: Screen::Audio } },
        Screen::Playlist => rsx! { ScreenPanel { screen: Screen::Playlist } },
        Screen::Naming => rsx! { ScreenPanel { screen: Screen::Naming } },
        Screen::Queue => rsx! { ScreenPanel { screen: Screen::Queue } },
        Screen::Library => rsx! { ScreenPanel { screen: Screen::Library } },
        Screen::Presets => rsx! { ScreenPanel { screen: Screen::Presets } },
        Screen::Advanced => rsx! { ScreenPanel { screen: Screen::Advanced } },
        Screen::Settings => rsx! { ScreenPanel { screen: Screen::Settings } },
        Screen::Error => rsx! { ScreenPanel { screen: Screen::Error } },
    }
}

#[component]
fn ScreenPanel(screen: Screen) -> Element {
    rsx! {
        section { class: "screen-grid",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "Workspace" }
                    h2 { "{screen.label()}" }
                    p { "{screen.caption()}" }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "Current screen" }
                    h3 { "{screen.label()}" }
                    p { "{screen.caption()}" }
                }
            }
        }
    }
}
