use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn LibraryView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let grid = (ctx.library_grid)();
    let completed: Vec<DownloadJob> = ctx
        .jobs
        .read()
        .iter()
        .filter(|job| job.status == JobStatus::Completed)
        .cloned()
        .collect();
    let completed_len = completed.len();
    let completed_count = i18n::t_with("completed_count", &[("count", completed_len.to_string())]);

    rsx! {
        section { class: "screen-grid inspector-layout",
            div { class: "main-column",
                div { class: "list-header",
                    div {
                        div { class: "eyebrow", "{i18n::t(\"library_completed_downloads\")}" }
                        h2 { "{Screen::Library.label()}" }
                    }
                    div { class: "table-tools",
                        button { class: if grid { "ghost-button active" } else { "ghost-button" }, onclick: move |_| ctx.library_grid.set(true), "{i18n::t(\"grid\")}" }
                        button { class: if grid { "ghost-button" } else { "ghost-button active" }, onclick: move |_| ctx.library_grid.set(false), "{i18n::t(\"list\")}" }
                    }
                }
                if completed.is_empty() {
                    EmptyState { title: i18n::t("empty_library_title"), message: i18n::t("empty_library_message") }
                } else {
                    div { class: if grid { "library-grid" } else { "library-grid list-mode" },
                        for job in completed {
                            LibraryCard { job }
                        }
                    }
                }
            }
            aside { class: "side-panel inspector-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"library_state\")}" }
                    h3 { "{completed_count}" }
                }
                StatBlock { label: i18n::t("storage"), value: ctx.settings().output_folder }
                StatBlock { label: i18n::t("source"), value: i18n::t("local_job_history") }
            }
        }
    }
}
