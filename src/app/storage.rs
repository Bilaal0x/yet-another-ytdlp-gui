use std::env;
use std::path::PathBuf;

use super::*;

pub(crate) fn load_preset_store() -> PresetStore {
    let path = preset_store_path();
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return PresetStore::defaults();
    };

    serde_json::from_str::<PresetStore>(&contents)
        .map(PresetStore::normalized)
        .unwrap_or_else(|_| PresetStore::defaults())
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

    let _ = save_preset_store(&store);
}

pub(crate) fn save_preset_store(store: &PresetStore) -> Result<(), String> {
    let path = preset_store_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|error| format!("failed to create storage folder: {error}"))?;
    }

    let contents = serde_json::to_string_pretty(store)
        .map_err(|error| format!("failed to serialize presets: {error}"))?;
    std::fs::write(path, contents).map_err(|error| format!("failed to write presets: {error}"))
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
}
