use super::*;

mod advanced;
mod audio;
mod error;
mod format;
mod home;
mod library;
mod naming;
mod playlist;
mod presets;
mod queue;
mod ready;
mod settings;

#[component]
pub(crate) fn ActiveView() -> Element {
    let ctx = use_context::<FetchContext>();

    match (ctx.screen)() {
        Screen::Home => rsx! { home::HomeView {} },
        Screen::Ready => rsx! { ready::ReadyView {} },
        Screen::Format => rsx! { format::FormatView {} },
        Screen::Audio => rsx! { audio::AudioView {} },
        Screen::Playlist => rsx! { playlist::PlaylistView {} },
        Screen::Naming => rsx! { naming::NamingView {} },
        Screen::Queue => rsx! { queue::QueueView {} },
        Screen::Library => rsx! { library::LibraryView {} },
        Screen::Presets => rsx! { presets::PresetsView {} },
        Screen::Advanced => rsx! { advanced::AdvancedView {} },
        Screen::Settings => rsx! { settings::SettingsView {} },
        Screen::Error => rsx! { error::ErrorView {} },
    }
}
