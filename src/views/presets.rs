use super::super::components::*;
use super::super::*;

#[component]
pub(super) fn PresetsView() -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let presets = (ctx.presets)();
    let selected_index = (ctx.active_preset)();
    let selected = ctx.active_preset();
    let selected_name = localized_preset_name(&selected.name);

    rsx! {
        section { class: "screen-grid inspector-layout",
            div { class: "main-column",
                div { class: "list-header",
                    div {
                        div { class: "eyebrow", "{i18n::t(\"saved_profiles\")}" }
                        h2 { "{Screen::Presets.label()}" }
                    }
                    button { class: "primary-button small", onclick: move |_| duplicate_preset(ctx), "{i18n::t(\"create_preset\")}" }
                }
                div { class: "preset-card-grid",
                    for (index, preset) in presets.iter().cloned().enumerate() {
                        {
                            let preset_name = localized_preset_name(&preset.name);
                            rsx! {
                        button {
                            class: if selected_index == index { "preset-card active" } else { "preset-card" },
                            onclick: move |_| {
                                ctx.active_preset.set(index);
                                persist_preset_store(ctx);
                            },
                            span { class: "format-title", "{preset_name}" }
                            span { "{i18n::t(\"preset_type\")}: {preset.kind.label()}" }
                            code { "{preset.format_rule}" }
                            span { "{i18n::t(\"preset_output\")}: {preset.output_template}" }
                            span { "{i18n::t(\"preset_flags\")}: {preset.extra_flags}" }
                        }
                            }
                        }
                    }
                }
            }
            aside { class: "side-panel editor-panel",
                div { class: "panel-heading compact",
                    div { class: "eyebrow", "{i18n::t(\"preset_editor\")}" }
                    h3 { "{selected_name}" }
                }
                EditablePresetField { label: i18n::t("preset_name"), field: "name".to_string(), value: selected.name }
                SelectPresetKind { value: selected.kind }
                EditablePresetField { label: i18n::t("preset_format_rule"), field: "format_rule".to_string(), value: selected.format_rule }
                EditablePresetField { label: i18n::t("audio_format"), field: "audio_format".to_string(), value: selected.audio_format }
                EditablePresetField { label: i18n::t("output_template"), field: "output_template".to_string(), value: selected.output_template }
                EditablePresetField { label: i18n::t("extra_flags"), field: "extra_flags".to_string(), value: selected.extra_flags }
                button { class: "primary-button full", onclick: move |_| apply_preset(ctx), "{i18n::t(\"use_preset\")}" }
            }
        }
    }
}
