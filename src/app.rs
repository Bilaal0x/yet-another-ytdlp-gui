use dioxus::prelude::*;

#[path = "components/mod.rs"]
mod components;
#[path = "views/mod.rs"]
mod views;

use components::{AppTitleBar, Sidebar, TopBar};
use views::ActiveView;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Home,
    Queue,
    Library,
    Presets,
    Settings,
    Advanced,
    Naming,
    Error,
}

impl Screen {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Screen::Home => "New Download",
            Screen::Queue => "Queue",
            Screen::Library => "Library",
            Screen::Presets => "Presets",
            Screen::Settings => "Settings",
            Screen::Advanced => "Advanced",
            Screen::Naming => "Save Location",
            Screen::Error => "Diagnostics",
        }
    }

    pub(crate) fn caption(self) -> &'static str {
        match self {
            Screen::Home => "Paste links and start analysis",
            Screen::Queue => "Track active jobs",
            Screen::Library => "Review completed downloads",
            Screen::Presets => "Manage reusable profiles",
            Screen::Settings => "Configure the desktop app",
            Screen::Advanced => "Inspect command options",
            Screen::Naming => "Choose output folders",
            Screen::Error => "Review troubleshooting details",
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct FetchContext {
    screen: Signal<Screen>,
}

#[component]
pub fn FetchApp() -> Element {
    let screen = use_signal(|| Screen::Home);

    use_context_provider(|| FetchContext { screen });

    rsx! {
        div { class: "window-root", dir: "ltr", "data-language": "en",
            AppTitleBar {}
            div { class: "app-shell",
                Sidebar {}
                main { class: "workspace",
                    TopBar {}
                    ActiveView {}
                }
            }
        }
    }
}
