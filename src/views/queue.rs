use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn QueueView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let jobs = (ctx.jobs)();
    let settings = ctx.settings();
    let pending = jobs
        .iter()
        .filter(|job| job.status == JobStatus::Queued || job.status == JobStatus::Failed)
        .count();
    let busy = (ctx.busy)();

    rsx! {
        section { class: "screen-grid inspector-layout",
            div { class: "main-column",
                div { class: "list-header",
                    div {
                        div { class: "eyebrow", "{i18n::t(\"queue_eyebrow\")}" }
                        h2 { "{Screen::Queue.label()}" }
                    }
                    div { class: "table-tools",
                        button { class: "ghost-button", onclick: move |_| clear_completed(ctx), "{i18n::t(\"clear_completed\")}" }
                        button { class: "ghost-button", onclick: move |_| ctx.screen.set(Screen::Home), "{i18n::t(\"add_new_url\")}" }
                        button {
                            class: "primary-button small",
                            disabled: pending == 0 || busy,
                            onclick: move |_| ctx.backend.send(BackendAction::StartQueue { settings: ctx.settings() }),
                            if busy { "{i18n::t(\"running\")}" } else { "{i18n::t(\"start_queue\")}" }
                        }
                    }
                }
                if jobs.is_empty() {
                    EmptyState { title: i18n::t("empty_queue_title"), message: i18n::t("empty_queue_message") }
                } else {
                    div { class: "queue-list",
                        for job in jobs {
                            DownloadRow { job }
                        }
                    }
                }
            }
            aside { class: "side-panel inspector-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"performance\")}" }
                    h3 { "{i18n::t(\"execution_controls\")}" }
                }
                RangeSetting { label: i18n::t("parallel_downloads"), field: "parallel_jobs".to_string(), value: settings.parallel_jobs, min: 1, max: 5 }
                RangeSetting { label: i18n::t("concurrent_fragments"), field: "concurrent_fragments".to_string(), value: settings.concurrent_fragments, min: 1, max: 12 }
                SelectField { label: i18n::t("speed_limit"), value: settings.speed_limit, options: vec!["Unlimited".to_string(), "10M".to_string(), "5M".to_string(), "1M".to_string()], target: "speed".to_string() }
                button { class: "secondary-button full", onclick: move |_| ctx.backend.send(BackendAction::RetryFailed { settings: ctx.settings() }), "{i18n::t(\"retry_failed\")}" }
            }
        }
    }
}
