use std::env;
use std::path::PathBuf;

use super::*;

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
    std::fs::read_to_string(locale_store_path())
        .ok()
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

fn app_data_dir() -> PathBuf {
    if let Some(path) = env::var_os("APPDATA").map(PathBuf::from) {
        return path.join("yaydlp");
    }

    if let Some(path) = env::var_os("USERPROFILE").map(PathBuf::from) {
        return path.join("AppData").join("Roaming").join("yaydlp");
    }

    PathBuf::from(".").join("yaydlp")
}
