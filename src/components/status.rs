use super::super::*;

#[component]
pub(crate) fn DependencyLine(item: DependencyItem) -> Element {
    let tone = if item.installed { "ok" } else { "missing" };
    let status = if item.installed {
        item.version.clone()
    } else {
        i18n::t("missing")
    };

    rsx! {
        div { class: "status-line",
            span { class: "status-label", "{item.name}" }
            span { class: "status-pill {tone}", "{status}" }
        }
    }
}

#[component]
pub(crate) fn DependencyCard(item: DependencyItem) -> Element {
    let tone = if item.installed { "ok" } else { "missing" };

    rsx! {
        div { class: "dependency-card {tone}",
            strong { "{item.name}" }
            span { "{item.detail}" }
        }
    }
}

#[component]
pub(crate) fn TroubleLine(title: String, detail: String) -> Element {
    rsx! {
        div { class: "trouble-line",
            strong { "{title}" }
            span { "{detail}" }
        }
    }
}

#[component]
pub(crate) fn CheckLine(text: String, ok: bool) -> Element {
    rsx! {
        div { class: "check-line",
            span { class: if ok { "check-dot" } else { "check-dot missing" } }
            "{text}"
        }
    }
}

#[component]
pub(crate) fn StatBlock(label: String, value: String) -> Element {
    rsx! {
        div { class: "stat-block",
            span { "{label}" }
            strong { "{value}" }
        }
    }
}

#[component]
pub(crate) fn EmptyState(title: String, message: String) -> Element {
    rsx! {
        div { class: "paste-panel empty-state",
            div { class: "panel-heading",
                div { class: "eyebrow", "{i18n::t(\"empty_state\")}" }
                h2 { "{title}" }
                p { "{message}" }
            }
        }
    }
}
