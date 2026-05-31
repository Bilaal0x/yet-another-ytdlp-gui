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
                        "{option}"
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
                oninput: move |event| {
                    let next = event.value();
                    update_selected_preset(ctx, &field, next);
                },
            }
        }
    }
}

#[component]
pub(crate) fn SelectPresetKind(value: DownloadType) -> Element {
    let ctx = use_context::<FetchContext>();
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
                    update_selected_preset_kind(ctx, kind);
                },
                option { value: "full_video", "{DownloadType::FullVideo.label()}" }
                option { value: "audio_only", "{DownloadType::AudioOnly.label()}" }
                option { value: "video_only", "{DownloadType::VideoOnly.label()}" }
            }
        }
    }
}

fn set_bool_setting(settings: &mut AppSettings, field: &str, value: bool) {
    match field {
        "write_subtitles" => settings.write_subtitles = value,
        "write_auto_subtitles" => settings.write_auto_subtitles = value,
        "write_thumbnail" => settings.write_thumbnail = value,
        "embed_thumbnail" => settings.embed_thumbnail = value,
        "add_metadata" => settings.add_metadata = value,
        "split_chapters" => settings.split_chapters = value,
        "keep_original" => settings.keep_original = value,
        "create_playlist_folders" => settings.create_playlist_folders = value,
        "add_playlist_index" => settings.add_playlist_index = value,
        "replace_unsafe_characters" => settings.replace_unsafe_characters = value,
        "prevent_overwrites" => settings.prevent_overwrites = value,
        "skip_existing" => settings.skip_existing = value,
        _ => {}
    }
}

fn set_i32_setting(settings: &mut AppSettings, field: &str, value: i32) {
    match field {
        "retries" => settings.retries = value,
        "parallel_jobs" => settings.parallel_jobs = value,
        "concurrent_fragments" => settings.concurrent_fragments = value,
        _ => {}
    }
}

fn download_type_value(value: DownloadType) -> &'static str {
    match value {
        DownloadType::FullVideo => "full_video",
        DownloadType::AudioOnly => "audio_only",
        DownloadType::VideoOnly => "video_only",
    }
}
