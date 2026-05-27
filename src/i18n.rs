use fluent_bundle::concurrent::FluentBundle;
use fluent_bundle::{FluentArgs, FluentResource, FluentValue};
use std::borrow::Cow;
use std::sync::{OnceLock, RwLock};
use unic_langid::LanguageIdentifier;

const AVAILABLE_LANGUAGES: &[(&str, &str)] = &[("en", "English"), ("de", "Deutsch")];

struct I18nState {
    bundle: FluentBundle<FluentResource>,
    fallback: Option<FluentBundle<FluentResource>>,
}

impl I18nState {
    fn new(lang: &str) -> Self {
        let bundle = make_bundle(lang)
            .or_else(|| make_bundle("en"))
            .expect("English locale must exist");
        let fallback = if lang != "en" {
            make_bundle("en")
        } else {
            None
        };

        Self { bundle, fallback }
    }

    fn translate(&self, key: &str, args: Option<&FluentArgs<'_>>) -> String {
        if let Some(value) = format_message(&self.bundle, key, args) {
            return value;
        }

        if let Some(fallback) = &self.fallback {
            if let Some(value) = format_message(fallback, key, args) {
                return value;
            }
        }

        key.to_string()
    }
}

static I18N: OnceLock<RwLock<I18nState>> = OnceLock::new();

fn state() -> &'static RwLock<I18nState> {
    I18N.get_or_init(|| RwLock::new(I18nState::new("en")))
}

pub fn init(lang: &str) {
    if I18N.set(RwLock::new(I18nState::new(lang))).is_err() {
        set_locale(lang);
    }
}

pub fn set_locale(lang: &str) {
    *state().write().unwrap() = I18nState::new(lang);
}

pub fn t(key: &str) -> String {
    state().read().unwrap().translate(key, None)
}

pub fn t_with(key: &str, args: &[(&str, String)]) -> String {
    let mut fluent_args: FluentArgs<'static> = FluentArgs::new();
    for (key, value) in args {
        fluent_args.set(
            Cow::Owned((*key).to_string()),
            FluentValue::String(Cow::Owned(value.clone())),
        );
    }

    state().read().unwrap().translate(key, Some(&fluent_args))
}

pub fn available_languages() -> &'static [(&'static str, &'static str)] {
    AVAILABLE_LANGUAGES
}

pub fn is_rtl() -> bool {
    t("is_rtl") == "true"
}

fn format_message(
    bundle: &FluentBundle<FluentResource>,
    key: &str,
    args: Option<&FluentArgs<'_>>,
) -> Option<String> {
    let message = bundle.get_message(key)?;
    let pattern = message.value()?;
    let mut errors = Vec::new();
    Some(
        bundle
            .format_pattern(pattern, args, &mut errors)
            .to_string(),
    )
}

fn make_bundle(lang: &str) -> Option<FluentBundle<FluentResource>> {
    let content = get_ftl_content(lang)?;
    let lang_id: LanguageIdentifier = lang.parse().ok()?;
    let mut bundle = FluentBundle::new_concurrent(vec![lang_id]);
    let resource = match FluentResource::try_new(content.to_string()) {
        Ok(resource) => resource,
        Err((resource, _)) => resource,
    };
    let _ = bundle.add_resource(resource);
    Some(bundle)
}

fn get_ftl_content(lang: &str) -> Option<&'static str> {
    match lang {
        "en" => Some(include_str!("../locales/en.ftl")),
        "de" => Some(include_str!("../locales/de.ftl")),
        _ => None,
    }
}
