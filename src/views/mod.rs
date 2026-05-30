use super::*;

mod audio;
mod format;
mod home;
mod library;
mod naming;
mod playlist;
mod queue;
mod ready;
mod settings;

#[component]
pub(crate) fn ActiveView() -> Element {
    let ctx = use_context::<FetchContext>();
    let _language = ctx.language();

    match (ctx.screen)() {
        Screen::Home => rsx! { home::HomeView {} },
        Screen::Ready => rsx! { ready::ReadyView {} },
        Screen::Format => rsx! { format::FormatView {} },
        Screen::Audio => rsx! { audio::AudioView {} },
        Screen::Playlist => rsx! { playlist::PlaylistView {} },
        Screen::Naming => rsx! { naming::NamingView {} },
        Screen::Queue => rsx! { queue::QueueView {} },
        Screen::Library => rsx! { library::LibraryView {} },
        Screen::Presets => rsx! { ScreenPanel { screen: Screen::Presets } },
        Screen::Advanced => rsx! { ScreenPanel { screen: Screen::Advanced } },
        Screen::Settings => rsx! { settings::SettingsView {} },
        Screen::Error => rsx! { ScreenPanel { screen: Screen::Error } },
    }
}

#[component]
fn ScreenPanel(screen: Screen) -> Element {
    let title = screen.label();
    let caption = screen.caption();

    rsx! {
        section { class: "screen-grid",
            div { class: "main-column",
                div { class: "panel-heading",
                    div { class: "eyebrow", "{i18n::t(\"workspace\")}" }
                    h2 { "{title}" }
                    p { "{caption}" }
                }
            }
            aside { class: "side-panel quiet-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"current_screen\")}" }
                    h3 { "{title}" }
                    p { "{caption}" }
                }
            }
        }
    }
}
