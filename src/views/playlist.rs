use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn PlaylistView() -> Element {
    let ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let analysis = (ctx.analysis)();

    match analysis {
        Some(result) => {
            let selected = result.items.iter().filter(|item| item.selected).count();
            let selected_count = i18n::t_with(
                "selection_count",
                &[
                    ("selected", selected.to_string()),
                    ("total", result.items.len().to_string()),
                ],
            );
            rsx! {
                section { class: "screen-grid inspector-layout",
                    div { class: "main-column",
                        div { class: "list-header",
                            div {
                                div { class: "eyebrow", "{i18n::t(\"analyzed_items\")}" }
                                h2 { "{result.source_label}" }
                            }
                            div { class: "table-tools",
                                button { class: "ghost-button", onclick: move |_| set_all_analysis_items(ctx, true), "{i18n::t(\"select_all\")}" }
                                button { class: "ghost-button", onclick: move |_| set_all_analysis_items(ctx, false), "{i18n::t(\"select_none\")}" }
                                button { class: "ghost-button", onclick: move |_| select_first_n(ctx, 10), "{i18n::t(\"first_10\")}" }
                            }
                        }
                        div { class: "playlist-table",
                            div { class: "playlist-head",
                                span { "" }
                                span { "{i18n::t(\"table_title\")}" }
                                span { "{i18n::t(\"table_duration\")}" }
                                span { "{i18n::t(\"table_formats\")}" }
                                span { "{i18n::t(\"table_estimate\")}" }
                            }
                            for (index, item) in result.items.iter().cloned().enumerate() {
                                PlaylistRow { index, item }
                            }
                        }
                        div { class: "option-grid three-col",
                            ToggleSetting { label: i18n::t("put_playlist_in_folder"), field: "create_playlist_folders".to_string(), value: ctx.settings().create_playlist_folders }
                            ToggleSetting { label: i18n::t("add_index_to_filenames"), field: "add_playlist_index".to_string(), value: ctx.settings().add_playlist_index }
                            ToggleSetting { label: i18n::t("skip_existing_files"), field: "skip_existing".to_string(), value: ctx.settings().skip_existing }
                        }
                    }
                    aside { class: "side-panel inspector-panel",
                        div { class: "panel-heading compact",
                            div { class: "eyebrow", "{i18n::t(\"selection\")}" }
                            h3 { "{selected_count}" }
                        }
                        StatBlock { label: i18n::t("source"), value: result.source_label }
                        StatBlock { label: i18n::t("output"), value: ctx.settings().output_folder }
                        button { class: "primary-button full", onclick: move |_| add_analysis_to_queue(ctx), "{i18n::t(\"add_selected_to_queue\")}" }
                        button { class: "secondary-button full", onclick: move |_| { select_first_n(ctx, 10); add_analysis_to_queue(ctx); }, "{i18n::t(\"download_first_10\")}" }
                    }
                }
            }
        }
        None => {
            rsx! { EmptyState { title: i18n::t("empty_no_playlist_title"), message: i18n::t("empty_no_playlist_message") } }
        }
    }
}
