use super::super::*;

#[component]
pub(crate) fn LanguageSelector(current_language: String) -> Element {
    let ctx = use_context::<FetchContext>();

    rsx! {
        select {
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
#[component]
pub(crate) fn SelectField(
    label: String,
    value: String,
    options: Vec<String>,
    target: String,
) -> Element {
    let mut ctx = use_context::<FetchContext>();

    rsx! {
        label { class: "setting-line",
            span { "{label}" }
            select {
                value: "{value}",
                onchange: move |event| {
                    let next = event.value();
                    match target.as_str() {
                        "container" => ctx.container.set(next),
                        "codec" => ctx.video_codec.set(next),
                        "resolution" => ctx.resolution_cap.set(next),
                        "audio_quality" => ctx.audio_quality.set(next),
                        "speed" => ctx.settings.with_mut(|settings| settings.speed_limit = next),
                        _ => {}
                    }
                },
                for option in options {
                    option {
                        value: "{option}",
                        "{localized_select_option(&option)}"
                    }
                }
            }
        }
    }
}
#[component]
pub(crate) fn ToggleSetting(label: String, field: String, value: bool) -> Element {
    let mut ctx = use_context::<FetchContext>();

    rsx! {
        label { class: "toggle-line",
            span { "{label}" }
            input {
                r#type: "checkbox",
                checked: value,
                onchange: move |_| ctx.settings.with_mut(|settings| set_bool_setting(settings, &field, !value)),
            }
        }
    }
}
#[component]
pub(crate) fn RangeSetting(
    label: String,
    field: String,
    value: i32,
    min: i32,
    max: i32,
) -> Element {
    let mut ctx = use_context::<FetchContext>();

    rsx! {
        label { class: "range-control",
            span { "{label}" }
            strong { "{value}" }
            input {
                r#type: "range",
                min: "{min}",
                max: "{max}",
                value: "{value}",
                oninput: move |event| {
                    if let Ok(next) = event.value().parse::<i32>() {
                        ctx.settings.with_mut(|settings| set_i32_setting(settings, &field, next));
                    }
                },
            }
        }
    }
}
#[component]
pub(crate) fn EditablePresetField(label: String, field: String, value: String) -> Element {
    let ctx = use_context::<FetchContext>();

    rsx! {
        label { class: "field-label",
            span { "{label}" }
            input {
                value: "{value}",
                oninput: move |event| update_selected_preset(ctx, &field, event.value()),
            }
        }
    }
}
#[component]
pub(crate) fn SelectPresetKind(value: DownloadType) -> Element {
    let mut ctx = use_context::<FetchContext>();
    let _language = ctx.language();
    let current = download_type_value(value);

    rsx! {
        label { class: "setting-line",
            span { "{i18n::t(\"download_type\")}" }
            select {
                value: "{current}",
                onchange: move |event| {
                    let kind = match event.value().as_str() {
                        "audio_only" => DownloadType::AudioOnly,
                        "video_only" => DownloadType::VideoOnly,
                        _ => DownloadType::FullVideo,
                    };
                    ctx.presets.with_mut(|presets| {
                        if let Some(preset) = presets.get_mut((ctx.active_preset)()) {
                            preset.kind = kind;
                        }
                    });
                    persist_preset_store(ctx);
                },
                option { value: "full_video", "{DownloadType::FullVideo.label()}" }
                option { value: "audio_only", "{DownloadType::AudioOnly.label()}" }
                option { value: "video_only", "{DownloadType::VideoOnly.label()}" }
            }
        }
    }
}
