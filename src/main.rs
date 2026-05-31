use dioxus::prelude::*;

mod app;

const MAIN_CSS: &str = include_str!("../assets/styling/main.css");
const TAILWIND_CSS: &str = include_str!("../assets/tailwind.css");

fn main() {
    #[cfg(feature = "desktop")]
    {
        use dioxus::desktop::{Config, LogicalSize, WindowBuilder};

        dioxus::LaunchBuilder::desktop()
            .with_cfg(
                Config::new().with_window(
                    WindowBuilder::new()
                        .with_title("yaydlp")
                        .with_decorations(false)
                        .with_inner_size(LogicalSize::new(1280.0, 860.0))
                        .with_min_inner_size(LogicalSize::new(1024.0, 720.0)),
                ),
            )
            .launch(App);
    }

    #[cfg(not(feature = "desktop"))]
    {
        dioxus::launch(App);
    }
}

#[component]
fn App() -> Element {
    rsx! {
        document::Style { "{MAIN_CSS}" }
        document::Style { "{TAILWIND_CSS}" }

        app::FetchApp {}
    }
}
