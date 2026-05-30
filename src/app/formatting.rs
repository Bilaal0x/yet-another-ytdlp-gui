use std::path::PathBuf;

use super::*;

pub(crate) fn preview_output_path(settings: &AppSettings) -> String {
    let example = settings
        .file_template
        .replace("%(playlist)s", "Playlist")
        .replace("%(playlist_index)s", "01")
        .replace("%(title)s", &i18n::t("example_title"))
        .replace("%(uploader)s", "Uploader")
        .replace("%(upload_date)s", "20260530")
        .replace("%(ext)s", "mp4");

    PathBuf::from(&settings.output_folder)
        .join(example)
        .display()
        .to_string()
}

pub(crate) fn template_is_valid(template: &str) -> bool {
    template.contains("%(title)s") && template.contains("%(ext)s")
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

pub(crate) fn audio_quality_label(quality: &str) -> String {
    match quality {
        "0" => "Best".to_string(),
        "320K" => "320 kbps".to_string(),
        "256K" => "256 kbps".to_string(),
        "192K" => "192 kbps".to_string(),
        other => other.to_string(),
    }
}

pub(crate) fn default_resolution_cap(format_label: &str) -> &'static str {
    match format_label {
        "MP4 720p" => "720p",
        "4K if available" => "2160p",
        "Best quality" | "Video only" => "Source",
        _ => "1080p",
    }
}

pub(crate) fn default_video_codec(format_label: &str) -> &'static str {
    match format_label {
        "Best quality" | "4K if available" | "Video only" => "Best",
        _ => "H.264",
    }
}

pub(crate) fn cap_to_number(cap: &str) -> Option<&str> {
    match cap {
        "720p" => Some("720"),
        "1080p" => Some("1080"),
        "1440p" => Some("1440"),
        "2160p" | "4K" => Some("2160"),
        "Source" | "No cap" => None,
        _ => Some("1080"),
    }
}

pub(crate) fn format_rule(format_label: &str, container: &str, codec: &str, cap: &str) -> String {
    let cap_filter = cap_to_number(cap)
        .map(|height| format!("[height<={height}]"))
        .unwrap_or_default();
    let codec = codec.to_ascii_lowercase();
    let container = container.to_ascii_lowercase();

    if format_label == "Video only" {
        return format!("bestvideo{cap_filter}");
    }

    if format_label == "Best quality" && cap_filter.is_empty() && codec == "best" {
        return "bestvideo+bestaudio/best".to_string();
    }

    if container == "webm" {
        return format!("bestvideo{cap_filter}[ext=webm]+bestaudio[ext=webm]/best{cap_filter}");
    }

    if codec == "av1" {
        return format!("bestvideo{cap_filter}[vcodec*=av01]+bestaudio/best{cap_filter}");
    }

    if codec == "h.264" || container == "mp4" {
        return format!(
            "bestvideo{cap_filter}[vcodec^=avc1][ext=mp4]+bestaudio[ext=m4a]/best{cap_filter}"
        );
    }

    format!("bestvideo{cap_filter}+bestaudio/best{cap_filter}")
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
        "No cap" | "Source" => i18n::t("format_source"),
        _ => cap.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_templates_with_title_and_extension() {
        assert!(template_is_valid("%(title)s.%(ext)s"));
        assert!(!template_is_valid("%(title)s"));
        assert!(!template_is_valid("%(id)s.%(ext)s"));
    }

    #[test]
    fn normalizes_audio_quality_labels() {
        assert_eq!(audio_quality_arg("Best"), "0");
        assert_eq!(audio_quality_arg("320 kbps"), "320K");
        assert_eq!(audio_quality_arg("128K"), "128K");
        assert_eq!(audio_quality_label("0"), "Best");
        assert_eq!(audio_quality_label("192K"), "192 kbps");
    }

    #[test]
    fn builds_format_rule_from_controls() {
        assert_eq!(
            format_rule("MP4 1080p", "MP4", "H.264", "1080p"),
            "bestvideo[height<=1080][vcodec^=avc1][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]"
        );
        assert_eq!(
            format_rule("4K if available", "MKV", "AV1", "2160p"),
            "bestvideo[height<=2160][vcodec*=av01]+bestaudio/best[height<=2160]"
        );
        assert_eq!(
            format_rule("Video only", "MKV", "Best", "Source"),
            "bestvideo"
        );
    }
}
