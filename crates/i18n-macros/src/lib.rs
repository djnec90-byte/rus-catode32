//! Compile-time translation lookup.
//!
//! `t!("key")` expands to a `&'static str` literal at compile time, looked up
//! against the JSON file for the active language feature. Lookup order:
//! `{lang}.json` → `en.json` → the key itself. The selected language is
//! determined by which `lang-*` feature is enabled (default: `lang-en`).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::OnceLock;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, LitStr};

fn active_lang() -> &'static str {
    if cfg!(feature = "lang-de") {
        "de"
    } else if cfg!(feature = "lang-es") {
        "es"
    } else if cfg!(feature = "lang-fr") {
        "fr"
    } else if cfg!(feature = "lang-it") {
        "it"
    } else if cfg!(feature = "lang-nl") {
        "nl"
    } else {
        "en"
    }
}

fn translations_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("translations")
}

fn load_json(path: PathBuf) -> HashMap<String, String> {
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("catode32-i18n-macros: failed to read {:?}: {}", path, e));
    serde_json::from_str(&raw)
        .unwrap_or_else(|e| panic!("catode32-i18n-macros: failed to parse {:?}: {}", path, e))
}

fn translations() -> &'static HashMap<String, String> {
    static CACHE: OnceLock<HashMap<String, String>> = OnceLock::new();
    CACHE.get_or_init(|| {
        let dir = translations_dir();
        let mut merged = load_json(dir.join("en.json"));
        let lang = active_lang();
        if lang != "en" {
            let lang_path = dir.join(format!("{}.json", lang));
            if lang_path.exists() {
                for (k, v) in load_json(lang_path) {
                    merged.insert(k, v);
                }
            }
        }
        merged
    })
}

/// Resolve a translation key at compile time.
///
/// `t!("Affection")` expands to the translated string literal for the active
/// language (or the key itself if no translation exists).
#[proc_macro]
pub fn t(input: TokenStream) -> TokenStream {
    let key_lit = parse_macro_input!(input as LitStr);
    let key = key_lit.value();
    let translated = translations()
        .get(&key)
        .cloned()
        .unwrap_or_else(|| key.clone());
    quote! { #translated }.into()
}
