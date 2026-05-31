use std::env;
use std::path::{Path, PathBuf};

use super::*;
use serde::{de::DeserializeOwned, Serialize};

pub(crate) fn load_settings() -> AppSettings {
    read_json_with_legacy(settings_store_path(), legacy_settings_store_path())
        .map(AppSettings::normalized)
        .unwrap_or_else(AppSettings::default)
}

pub(crate) fn persist_settings(settings: &AppSettings) {
    if let Err(error) = save_settings(settings) {
        eprintln!(
            "Could not save settings to {}: {error}",
            settings_store_path().display()
        );
    }
}

pub(crate) fn save_settings(settings: &AppSettings) -> Result<(), String> {
    let settings = settings.clone().normalized();
    save_json(settings_store_path(), &settings)?;
    save_language(&settings.language);
    Ok(())
}

pub(crate) fn settings_store_path() -> PathBuf {
    app_data_dir().join("settings.json")
}

pub(crate) fn load_library() -> Vec<DownloadJob> {
    read_json_with_legacy::<LibraryStore>(library_store_path(), legacy_library_store_path())
        .map(LibraryStore::normalized)
        .map(|store| store.jobs)
        .unwrap_or_default()
}

pub(crate) fn persist_library(jobs: &[DownloadJob]) {
    if let Err(error) = save_library(jobs) {
        eprintln!(
            "Could not save library to {}: {error}",
            library_store_path().display()
        );
    }
}

pub(crate) fn save_library(jobs: &[DownloadJob]) -> Result<(), String> {
    let store = LibraryStore {
        version: 1,
        jobs: jobs
            .iter()
            .filter(|job| job.status == JobStatus::Completed)
            .cloned()
            .collect(),
    }
    .normalized();

    save_json(library_store_path(), &store)
}

pub(crate) fn library_store_path() -> PathBuf {
    app_data_dir().join("library.json")
}

pub(crate) fn next_job_id_after(jobs: &[DownloadJob]) -> u64 {
    jobs.iter()
        .map(|job| job.id)
        .max()
        .and_then(|id| id.checked_add(1))
        .unwrap_or(1)
}

pub(crate) fn load_preset_store() -> PresetStore {
    read_json_with_legacy(preset_store_path(), legacy_preset_store_path())
        .map(PresetStore::normalized)
        .unwrap_or_else(PresetStore::defaults)
}

pub(crate) fn persist_preset_store(ctx: FetchContext) {
    let presets = (ctx.presets)();
    let active_index = if presets.is_empty() {
        0
    } else {
        (ctx.active_preset)().min(presets.len() - 1)
    };
    let store = PresetStore {
        version: 1,
        active_index,
        presets,
    }
    .normalized();

    if let Err(error) = save_preset_store(&store) {
        eprintln!(
            "Could not save presets to {}: {error}",
            preset_store_path().display()
        );
    }
}

pub(crate) fn save_preset_store(store: &PresetStore) -> Result<(), String> {
    save_json(preset_store_path(), store)
}

pub(crate) fn preset_store_path() -> PathBuf {
    app_data_dir().join("presets.json")
}

pub(crate) fn set_language(mut ctx: FetchContext, language: String) {
    if !language_is_available(&language) {
        return;
    }

    i18n::set_locale(&language);
    ctx.settings
        .with_mut(|settings| settings.language = language.clone());
    save_language(&language);
}

pub(crate) fn default_language() -> String {
    read_text_with_legacy(locale_store_path(), legacy_locale_store_path())
        .map(|value| value.trim().to_string())
        .filter(|value| language_is_available(value))
        .unwrap_or_else(|| "en".to_string())
}

pub(crate) fn language_is_available(language: &str) -> bool {
    i18n::available_languages()
        .iter()
        .any(|(code, _)| *code == language)
}

pub(crate) fn save_language(language: &str) {
    let path = locale_store_path();
    if let Some(parent) = path.parent() {
        if std::fs::create_dir_all(parent).is_err() {
            return;
        }
    }

    let _ = std::fs::write(path, language);
}

pub(crate) fn locale_store_path() -> PathBuf {
    app_data_dir().join("locale.txt")
}

pub(crate) fn default_download_folder() -> String {
    env::var("USERPROFILE")
        .map(|home| format!("{home}\\Downloads\\YAYDLP"))
        .unwrap_or_else(|_| "downloads".to_string())
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct LibraryStore {
    version: u32,
    jobs: Vec<DownloadJob>,
}

impl LibraryStore {
    fn normalized(mut self) -> Self {
        self.version = 1;
        self.jobs.retain(|job| job.status == JobStatus::Completed);

        for job in &mut self.jobs {
            job.progress = 100.0;
            job.speed = "-".to_string();
            job.eta = "-".to_string();
            job.error = None;
        }

        self
    }
}

fn read_json_with_legacy<T>(path: PathBuf, legacy_path: PathBuf) -> Option<T>
where
    T: DeserializeOwned,
{
    read_json(&path).ok().or_else(|| {
        if legacy_path == path {
            None
        } else {
            read_json(&legacy_path).ok()
        }
    })
}

fn read_json<T>(path: &Path) -> Result<T, String>
where
    T: DeserializeOwned,
{
    let contents = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

fn save_json<T>(path: PathBuf, value: &T) -> Result<(), String>
where
    T: Serialize,
{
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create storage folder: {error}"))?;
    }

    let contents = serde_json::to_string_pretty(value)
        .map_err(|error| format!("failed to serialize: {error}"))?;
    std::fs::write(&path, contents)
        .map_err(|error| format!("failed to write {}: {error}", path.display()))
}

fn read_text_with_legacy(path: PathBuf, legacy_path: PathBuf) -> Option<String> {
    std::fs::read_to_string(&path).ok().or_else(|| {
        if legacy_path == path {
            None
        } else {
            std::fs::read_to_string(legacy_path).ok()
        }
    })
}

fn legacy_settings_store_path() -> PathBuf {
    legacy_app_data_dir().join("settings.json")
}

fn legacy_library_store_path() -> PathBuf {
    legacy_app_data_dir().join("library.json")
}

fn legacy_preset_store_path() -> PathBuf {
    legacy_app_data_dir().join("presets.json")
}

fn legacy_locale_store_path() -> PathBuf {
    legacy_app_data_dir().join("locale.txt")
}

fn app_data_dir() -> PathBuf {
    if let Some(path) = env::var_os("APPDATA").map(PathBuf::from) {
        return path.join("YAMDL");
    }

    if let Some(path) = env::var_os("USERPROFILE").map(PathBuf::from) {
        return path.join("AppData").join("Roaming").join("YAMDL");
    }

    PathBuf::from(".").join("YAMDL")
}

fn legacy_app_data_dir() -> PathBuf {
    if let Some(path) = env::var_os("APPDATA").map(PathBuf::from) {
        return path.join("yaydlp");
    }

    if let Some(path) = env::var_os("USERPROFILE").map(PathBuf::from) {
        return path.join("AppData").join("Roaming").join("yaydlp");
    }

    PathBuf::from(".").join("yaydlp")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_empty_preset_store_to_defaults() {
        let store = PresetStore {
            version: 1,
            active_index: 42,
            presets: Vec::new(),
        }
        .normalized();

        assert_eq!(store.active_index, 0);
        assert!(!store.presets.is_empty());
    }

    #[test]
    fn normalizes_out_of_range_active_preset() {
        let store = PresetStore {
            version: 1,
            active_index: 99,
            presets: Preset::defaults(),
        }
        .normalized();

        assert_eq!(store.active_index, 0);
    }

    #[test]
    fn normalizes_invalid_settings_values() {
        let settings = AppSettings {
            language: "missing".to_string(),
            output_folder: String::new(),
            file_template: String::new(),
            subtitle_languages: String::new(),
            retries: -4,
            parallel_jobs: 0,
            concurrent_fragments: 0,
            speed_limit: String::new(),
            ..Default::default()
        }
        .normalized();

        assert!(language_is_available(&settings.language));
        assert!(!settings.output_folder.is_empty());
        assert!(!settings.file_template.is_empty());
        assert!(!settings.subtitle_languages.is_empty());
        assert_eq!(settings.retries, 0);
        assert_eq!(settings.parallel_jobs, 1);
        assert_eq!(settings.concurrent_fragments, 1);
        assert_eq!(settings.speed_limit, "Unlimited");
    }

    #[test]
    fn library_store_keeps_only_completed_jobs() {
        let store = LibraryStore {
            version: 99,
            jobs: vec![
                test_job(1, JobStatus::Completed),
                test_job(2, JobStatus::Failed),
            ],
        }
        .normalized();

        assert_eq!(store.version, 1);
        assert_eq!(store.jobs.len(), 1);
        assert_eq!(store.jobs[0].id, 1);
        assert_eq!(store.jobs[0].progress, 100.0);
        assert_eq!(store.jobs[0].speed, "-");
        assert_eq!(store.jobs[0].eta, "-");
        assert!(store.jobs[0].error.is_none());
    }

    #[test]
    fn next_job_id_follows_loaded_library() {
        let jobs = vec![
            test_job(7, JobStatus::Completed),
            test_job(2, JobStatus::Completed),
        ];

        assert_eq!(next_job_id_after(&jobs), 8);
    }

    fn test_job(id: u64, status: JobStatus) -> DownloadJob {
        DownloadJob {
            id,
            title: format!("Job {id}"),
            source_url: "https://example.com/video".to_string(),
            thumbnail: String::new(),
            download_type: DownloadType::FullVideo,
            format_label: "MP4 1080p".to_string(),
            audio_format: "mp3".to_string(),
            audio_quality: "320K".to_string(),
            container: "mp4".to_string(),
            video_codec: "H.264".to_string(),
            resolution_cap: "1080p".to_string(),
            output_folder: "downloads".to_string(),
            output_template: "%(title)s.%(ext)s".to_string(),
            command_args: Vec::new(),
            command_display: "yt-dlp https://example.com/video".to_string(),
            status,
            progress: 42.0,
            speed: "1MiB/s".to_string(),
            eta: "00:01".to_string(),
            step: "Downloading".to_string(),
            output_hint: "downloads/video.mp4".to_string(),
            log: Vec::new(),
            error: Some(AppError::new("error", "message", "debug")),
        }
    }
}
