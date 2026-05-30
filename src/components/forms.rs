use super::super::*;

#[component]
pub(crate) fn LanguageSelector(current_language: String) -> Element {
    let ctx = use_context::<FetchContext>();

    rsx! {
        select {
            title: "{i18n::t(\"language\")}",
            value: "{current_language}",
            onchange: move |event| set_language(ctx, event.value()),
            for (code, name) in i18n::available_languages() {
                option {
                    value: *code,
                    selected: *code == current_language.as_str(),
                    "{name}"
                }
            }
        }
    }
}
