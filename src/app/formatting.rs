use serde_json::Value;
use std::path::PathBuf;

use super::*;

pub(crate) fn preview_output_path(settings: &AppSettings) -> String {
    let example = settings
        .file_template
        .replace("%(playlist)s", "Playlist")
        .replace("%(playlist_index)s", "01")
        .replace("%(title)s", &i18n::t("example_title"))
        .replace("%(uploader)s", "Uploader")
        .replace("%(upload_date)s", "20260515")
        .replace("%(ext)s", "mp4");
    PathBuf::from(&settings.output_folder)
        .join(example)
        .display()
        .to_string()
}

pub(crate) fn template_is_valid(template: &str) -> bool {
    template.contains("%(title)") && template.contains("%(ext)")
}

pub(crate) fn audio_format_detail(format: &str) -> String {
    match format {
        "FLAC" => i18n::t("audio_detail_flac"),
        "M4A" => i18n::t("audio_detail_m4a"),
        "Opus" => i18n::t("audio_detail_opus"),
        "WAV" => i18n::t("audio_detail_wav"),
        _ => i18n::t("audio_detail_default"),
    }
}

pub(crate) fn audio_quality_arg(quality: &str) -> &str {
    match quality {
        "Best" => "0",
        "320 kbps" => "320K",
        "256 kbps" => "256K",
        "192 kbps" => "192K",
        other => other,
    }
}

pub(crate) fn cap_to_number(cap: &str) -> &str {
    match cap {
        "720p" => "720",
        "4K" => "2160",
        _ => "1080",
    }
}

pub(crate) fn format_duration(seconds: f64) -> String {
    let total = seconds.round().max(0.0) as u64;
    let hours = total / 3600;
    let minutes = (total % 3600) / 60;
    let seconds = total % 60;
    if hours > 0 {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes}:{seconds:02}")
    }
}

pub(crate) fn estimate_size(json: &Value) -> String {
    let bytes = json
        .get("filesize_approx")
        .or_else(|| json.get("filesize"))
        .and_then(Value::as_u64)
        .or_else(|| {
            json.get("formats")
                .and_then(Value::as_array)
                .and_then(|formats| {
                    formats
                        .iter()
                        .filter_map(|format| format.get("filesize").and_then(Value::as_u64))
                        .max()
                })
        });

    match bytes {
        Some(bytes) if bytes > 1_000_000_000 => format!("{:.1} GB", bytes as f64 / 1_000_000_000.0),
        Some(bytes) if bytes > 1_000_000 => format!("{:.1} MB", bytes as f64 / 1_000_000.0),
        Some(bytes) if bytes > 0 => format!("{:.1} KB", bytes as f64 / 1_000.0),
        _ => i18n::t("unknown"),
    }
}

pub(crate) fn parse_percent(line: &str) -> Option<f32> {
    let percent_index = line.find('%')?;
    let before = &line[..percent_index];
    let number: String = before
        .chars()
        .rev()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>()
        .chars()
        .rev()
        .collect();
    number.parse::<f32>().ok()
}

pub(crate) fn parse_between(line: &str, start: &str, end: &str) -> Option<String> {
    let start_index = line.find(start)? + start.len();
    let rest = &line[start_index..];
    let end_index = rest.find(end)?;
    Some(rest[..end_index].trim().to_string())
}

pub(crate) fn parse_after(line: &str, marker: &str) -> Option<String> {
    let index = line.find(marker)? + marker.len();
    Some(line[index..].trim().to_string())
}

pub(crate) fn process_destination(line: &str) -> Option<String> {
    let destination = line
        .strip_prefix("[download] Destination:")
        .or_else(|| line.strip_prefix("[ExtractAudio] Destination:"))?
        .trim()
        .trim_matches('"');

    if destination.is_empty() {
        None
    } else {
        Some(destination.to_string())
    }
}

pub(crate) fn short_version(version: &str) -> String {
    version
        .split_whitespace()
        .next()
        .map(ToString::to_string)
        .unwrap_or_else(|| i18n::t("installed"))
}

pub(crate) fn first_nonempty_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map(ToString::to_string)
}

pub(crate) fn first_error_line(text: &str) -> Option<String> {
    text.lines()
        .map(str::trim)
        .find(|line| {
            let lower = line.to_ascii_lowercase();
            lower.starts_with("error:")
                || lower.contains("unable to")
                || lower.contains("failed")
                || lower.contains("errno")
        })
        .map(ToString::to_string)
}

pub(crate) fn friendly_suggestion(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("ffmpeg") {
        i18n::t("suggestion_ffmpeg")
    } else if lower.contains("yt-dlp") {
        i18n::t("suggestion_ytdlp")
    } else if lower.contains("cookies") || lower.contains("login") || lower.contains("private") {
        i18n::t("suggestion_cookies")
    } else if lower.contains("format") {
        i18n::t("suggestion_format")
    } else if lower.contains("network") || lower.contains("timed out") {
        i18n::t("suggestion_network")
    } else {
        i18n::t("suggestion_default")
    }
}

pub(crate) fn localized_preset_name(name: &str) -> String {
    if let Some(base) = name.strip_suffix(" copy") {
        return format!(
            "{} {}",
            localized_preset_name(base),
            i18n::t("preset_copy_suffix")
        );
    }

    match name {
        "YouTube MP4 1080p" => i18n::t("preset_youtube_mp4_1080p"),
        "Audio MP3 320" => i18n::t("preset_audio_mp3_320"),
        "Archive playlist" => i18n::t("preset_archive_playlist"),
        "4K HDR" => i18n::t("preset_4k_hdr"),
        "Small file 720p" => i18n::t("preset_small_file_720p"),
        _ => name.to_string(),
    }
}

pub(crate) fn localized_preset_detail(detail: &str) -> String {
    if let Some((prefix, format_label)) = detail.split_once(" / ") {
        format!("{prefix} / {}", localized_format_label(format_label))
    } else {
        detail.to_string()
    }
}

pub(crate) fn localized_format_label(label: &str) -> String {
    match label {
        "Best audio" => i18n::t("format_label_best_audio"),
        "Best quality" => i18n::t("format_label_best_quality"),
        "4K if available" => i18n::t("format_label_4k_if_available"),
        "Video only" => i18n::t("format_label_video_only"),
        _ => label.to_string(),
    }
}

pub(crate) fn localized_format_cap(cap: &str) -> String {
    match cap {
        "No cap" => i18n::t("format_no_cap"),
        "Source" => i18n::t("format_source"),
        _ => cap.to_string(),
    }
}

pub(crate) fn localized_select_option(option: &str) -> String {
    match option {
        "Best" => i18n::t("option_best"),
        "Unlimited" => i18n::t("option_unlimited"),
        _ => option.to_string(),
    }
}

pub(crate) fn localized_job_step(step: &str) -> String {
    match step {
        "Queued" => i18n::t("job_step_queued"),
        "Queued for retry" => i18n::t("job_step_queued_retry"),
        "Starting yt-dlp" => i18n::t("job_step_starting"),
        "Downloading" => i18n::t("job_step_downloading"),
        "Merging" => i18n::t("job_step_merging"),
        "Extracting audio" => i18n::t("job_step_extracting_audio"),
        "Embedding metadata" => i18n::t("job_step_embedding_metadata"),
        "Completed" => i18n::t("job_step_completed"),
        _ => step.to_string(),
    }
}

pub(crate) fn download_type_value(value: DownloadType) -> &'static str {
    match value {
        DownloadType::FullVideo => "full_video",
        DownloadType::AudioOnly => "audio_only",
        DownloadType::VideoOnly => "video_only",
    }
}
