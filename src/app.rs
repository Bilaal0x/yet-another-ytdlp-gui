#![allow(dead_code)]

use dioxus::prelude::*;
use serde::{Deserialize, Serialize};

#[path = "components/mod.rs"]
mod components;
#[path = "views/mod.rs"]
mod views;

use components::{AppTitleBar, Sidebar, TopBar};
use views::ActiveView;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Screen {
    Home,
    Ready,
    Format,
    Audio,
    Playlist,
    Naming,
    Queue,
    Library,
    Presets,
    Advanced,
    Settings,
    Error,
}

impl Screen {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Screen::Home => "New Download",
            Screen::Ready => "Ready",
            Screen::Format => "Format",
            Screen::Audio => "Audio",
            Screen::Playlist => "Batch Review",
            Screen::Naming => "Save Location",
            Screen::Queue => "Queue",
            Screen::Library => "Library",
            Screen::Presets => "Presets",
            Screen::Advanced => "Advanced",
            Screen::Settings => "Settings",
            Screen::Error => "Diagnostics",
        }
    }

    pub(crate) fn caption(self) -> &'static str {
        match self {
            Screen::Home => "Paste links and start analysis",
            Screen::Ready => "Review analyzed media",
            Screen::Format => "Choose video format",
            Screen::Audio => "Configure audio extraction",
            Screen::Playlist => "Select playlist entries",
            Screen::Naming => "Choose output folders",
            Screen::Queue => "Track active jobs",
            Screen::Library => "Review completed downloads",
            Screen::Presets => "Manage reusable profiles",
            Screen::Advanced => "Inspect command options",
            Screen::Settings => "Configure the desktop app",
            Screen::Error => "Review troubleshooting details",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) enum DownloadType {
    FullVideo,
    AudioOnly,
    VideoOnly,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

#[derive(Clone, PartialEq)]
pub(crate) struct AppSettings {
    language: String,
    output_folder: String,
    file_template: String,
    subtitle_languages: String,
    write_subtitles: bool,
    write_auto_subtitles: bool,
    write_thumbnail: bool,
    embed_thumbnail: bool,
    add_metadata: bool,
    split_chapters: bool,
    keep_original: bool,
    create_playlist_folders: bool,
    add_playlist_index: bool,
    replace_unsafe_characters: bool,
    prevent_overwrites: bool,
    skip_existing: bool,
    cookie_file: String,
    proxy: String,
    retries: i32,
    parallel_jobs: i32,
    concurrent_fragments: i32,
    speed_limit: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            output_folder: "downloads".to_string(),
            file_template: "%(playlist)s/%(playlist_index)s - %(title)s.%(ext)s".to_string(),
            subtitle_languages: "en,de,und".to_string(),
            write_subtitles: false,
            write_auto_subtitles: false,
            write_thumbnail: true,
            embed_thumbnail: true,
            add_metadata: true,
            split_chapters: false,
            keep_original: false,
            create_playlist_folders: true,
            add_playlist_index: true,
            replace_unsafe_characters: true,
            prevent_overwrites: true,
            skip_existing: true,
            cookie_file: String::new(),
            proxy: String::new(),
            retries: 5,
            parallel_jobs: 2,
            concurrent_fragments: 4,
            speed_limit: "Unlimited".to_string(),
        }
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct Preset {
    name: String,
    kind: DownloadType,
    format_label: String,
    format_rule: String,
    audio_format: String,
    audio_quality: String,
    container: String,
    output_template: String,
    extra_flags: String,
}

impl Preset {
    fn defaults() -> Vec<Self> {
        vec![
            Self {
                name: "YouTube MP4 1080p".to_string(),
                kind: DownloadType::FullVideo,
                format_label: "MP4 1080p".to_string(),
                format_rule:
                    "bestvideo[height<=1080][ext=mp4]+bestaudio[ext=m4a]/best[height<=1080]"
                        .to_string(),
                audio_format: "mp3".to_string(),
                audio_quality: "320K".to_string(),
                container: "mp4".to_string(),
                output_template: "%(title)s.%(ext)s".to_string(),
                extra_flags: "--embed-thumbnail --add-metadata".to_string(),
            },
            Self {
                name: "Audio MP3 320".to_string(),
                kind: DownloadType::AudioOnly,
                format_label: "Best audio".to_string(),
                format_rule: "bestaudio/best".to_string(),
                audio_format: "mp3".to_string(),
                audio_quality: "320K".to_string(),
                container: "mp3".to_string(),
                output_template: "%(artist,uploader)s - %(title)s.%(ext)s".to_string(),
                extra_flags: "--extract-audio --embed-thumbnail --add-metadata".to_string(),
            },
            Self {
                name: "Archive playlist".to_string(),
                kind: DownloadType::FullVideo,
                format_label: "Best quality".to_string(),
                format_rule: "bestvideo+bestaudio/best".to_string(),
                audio_format: "mp3".to_string(),
                audio_quality: "0".to_string(),
                container: "mkv".to_string(),
                output_template: "%(playlist)s/%(playlist_index)s - %(title)s.%(ext)s".to_string(),
                extra_flags: "--download-archive archive.txt --no-overwrites".to_string(),
            },
            Self {
                name: "4K HDR".to_string(),
                kind: DownloadType::FullVideo,
                format_label: "4K if available".to_string(),
                format_rule: "bestvideo[height<=2160]+bestaudio/best[height<=2160]".to_string(),
                audio_format: "mp3".to_string(),
                audio_quality: "0".to_string(),
                container: "mkv".to_string(),
                output_template: "%(title)s.%(ext)s".to_string(),
                extra_flags: "--embed-thumbnail --add-metadata".to_string(),
            },
            Self {
                name: "Small file 720p".to_string(),
                kind: DownloadType::FullVideo,
                format_label: "MP4 720p".to_string(),
                format_rule: "bestvideo[height<=720][ext=mp4]+bestaudio[ext=m4a]/best[height<=720]"
                    .to_string(),
                audio_format: "mp3".to_string(),
                audio_quality: "192K".to_string(),
                container: "mp4".to_string(),
                output_template: "%(title)s.%(ext)s".to_string(),
                extra_flags: "--no-overwrites".to_string(),
            },
        ]
    }
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub(crate) struct PresetStore {
    version: u32,
    active_index: usize,
    presets: Vec<Preset>,
}

impl PresetStore {
    fn defaults() -> Self {
        Self {
            version: 1,
            active_index: 0,
            presets: Preset::defaults(),
        }
    }

    fn normalized(mut self) -> Self {
        if self.presets.is_empty() {
            self.presets = Preset::defaults();
            self.active_index = 0;
        }

        if self.active_index >= self.presets.len() {
            self.active_index = 0;
        }

        self
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct MediaItem {
    title: String,
    uploader: String,
    url: String,
    duration: String,
    thumbnail: String,
    format_count: usize,
    estimated_size: String,
    selected: bool,
}

#[derive(Clone, PartialEq)]
pub(crate) struct AnalysisResult {
    source_label: String,
    items: Vec<MediaItem>,
    command: String,
    warnings: Vec<String>,
}

#[derive(Clone, PartialEq)]
pub(crate) struct DownloadJob {
    id: u64,
    title: String,
    source_url: String,
    thumbnail: String,
    download_type: DownloadType,
    format_label: String,
    audio_format: String,
    container: String,
    output_folder: String,
    output_template: String,
    command_display: String,
    status: JobStatus,
    progress: f32,
    speed: String,
    eta: String,
    step: String,
    output_hint: String,
    log: Vec<String>,
    error: Option<AppError>,
}

#[derive(Clone, PartialEq)]
pub(crate) struct DependencyReport {
    ytdlp: DependencyItem,
    ffmpeg: DependencyItem,
}

impl Default for DependencyReport {
    fn default() -> Self {
        Self {
            ytdlp: DependencyItem::pending("yt-dlp"),
            ffmpeg: DependencyItem::pending("FFmpeg"),
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct DependencyItem {
    name: String,
    installed: bool,
    version: String,
    detail: String,
}

impl DependencyItem {
    fn pending(name: &str) -> Self {
        Self {
            name: name.to_string(),
            installed: false,
            version: "Checking".to_string(),
            detail: "Not checked yet".to_string(),
        }
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct AppError {
    title: String,
    message: String,
    suggestion: String,
    debug: String,
}

impl AppError {
    fn new(title: impl Into<String>, message: impl Into<String>, debug: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            suggestion: String::new(),
            debug: debug.into(),
        }
    }
}

pub(crate) enum BackendAction {
    RefreshDependencies,
    Analyze {
        urls: Vec<String>,
        settings: AppSettings,
    },
    StartQueue {
        settings: AppSettings,
    },
    RetryFailed {
        settings: AppSettings,
    },
}

#[derive(Clone, Copy)]
pub(crate) struct FetchContext {
    screen: Signal<Screen>,
    download_type: Signal<DownloadType>,
    url_text: Signal<String>,
    selected_format: Signal<String>,
    selected_audio_format: Signal<String>,
    audio_quality: Signal<String>,
    container: Signal<String>,
    video_codec: Signal<String>,
    resolution_cap: Signal<String>,
    active_preset: Signal<usize>,
    settings: Signal<AppSettings>,
    presets: Signal<Vec<Preset>>,
    analysis: Signal<Option<AnalysisResult>>,
    jobs: Signal<Vec<DownloadJob>>,
    dependencies: Signal<DependencyReport>,
    busy: Signal<bool>,
    analysis_running: Signal<bool>,
    analysis_cancel_token: Signal<u64>,
    show_cli: Signal<bool>,
    show_debug: Signal<bool>,
    library_grid: Signal<bool>,
    last_error: Signal<Option<AppError>>,
    next_job_id: Signal<u64>,
}

impl FetchContext {
    fn active_preset(&self) -> Preset {
        let presets = (self.presets)();
        presets
            .get((self.active_preset)())
            .cloned()
            .unwrap_or_else(|| Preset::defaults().remove(0))
    }
}

#[component]
pub fn FetchApp() -> Element {
    let screen = use_signal(|| Screen::Home);
    let download_type = use_signal(|| DownloadType::FullVideo);
    let url_text = use_signal(String::new);
    let selected_format = use_signal(|| "MP4 1080p".to_string());
    let selected_audio_format = use_signal(|| "MP3".to_string());
    let audio_quality = use_signal(|| "320 kbps".to_string());
    let container = use_signal(|| "MP4".to_string());
    let video_codec = use_signal(|| "H.264".to_string());
    let resolution_cap = use_signal(|| "1080p".to_string());
    let active_preset = use_signal(|| 0usize);
    let settings = use_signal(AppSettings::default);
    let presets = use_signal(Preset::defaults);
    let analysis = use_signal(|| None::<AnalysisResult>);
    let jobs = use_signal(Vec::<DownloadJob>::new);
    let dependencies = use_signal(DependencyReport::default);
    let busy = use_signal(|| false);
    let analysis_running = use_signal(|| false);
    let analysis_cancel_token = use_signal(|| 0u64);
    let show_cli = use_signal(|| false);
    let show_debug = use_signal(|| false);
    let library_grid = use_signal(|| true);
    let last_error = use_signal(|| None::<AppError>);
    let next_job_id = use_signal(|| 1u64);

    use_context_provider(|| FetchContext {
        screen,
        download_type,
        url_text,
        selected_format,
        selected_audio_format,
        audio_quality,
        container,
        video_codec,
        resolution_cap,
        active_preset,
        settings,
        presets,
        analysis,
        jobs,
        dependencies,
        busy,
        analysis_running,
        analysis_cancel_token,
        show_cli,
        show_debug,
        library_grid,
        last_error,
        next_job_id,
    });

    rsx! {
        div { class: "window-root", dir: "ltr", "data-language": "en",
            AppTitleBar {}
            div { class: "app-shell",
                Sidebar {}
                main { class: "workspace",
                    TopBar {}
                    ActiveView {}
                }
            }
        }
    }
}
