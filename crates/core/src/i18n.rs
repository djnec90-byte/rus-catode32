//! Translation helpers used across scenes. `t!()` itself comes from the
//! `catode32-i18n-macros` crate and is re-exported from `lib.rs`; this module
//! provides the runtime tools — `to_lower` and `substitute` — needed when a
//! translated string has to be lowercased for embedding or when its
//! placeholder set varies between languages.

use heapless::String;

/// Copy `src` into a buffer, lowercased. Used for embedded forms ("kibble"
/// in "favorite meal is kibble") so the lowercase derives from the translation
/// rather than being hardcoded in English.
pub fn to_lower<const N: usize>(src: &str) -> String<N> {
    let mut out: String<N> = String::new();
    for c in src.chars() {
        for lc in c.to_lowercase() {
            let _ = out.push(lc);
        }
    }
    out
}

/// Runtime template substitution for translated strings whose placeholder set
/// varies between languages. Unlike `write!`, missing or unused names don't
/// error: unknown `{name}` passes through unchanged, unused entries in `subs`
/// are silently ignored.
pub fn substitute<const N: usize>(out: &mut String<N>, template: &str, subs: &[(&str, &str)]) {
    let mut remaining = template;
    while let Some(open) = remaining.find('{') {
        let (head, tail) = remaining.split_at(open);
        let _ = out.push_str(head);
        if let Some(close) = tail.find('}') {
            let key = &tail[1..close];
            let mut matched = false;
            for &(k, v) in subs {
                if k == key {
                    let _ = out.push_str(v);
                    matched = true;
                    break;
                }
            }
            if !matched {
                let _ = out.push_str(&tail[..=close]);
            }
            remaining = &tail[close + 1..];
        } else {
            let _ = out.push_str(tail);
            remaining = "";
        }
    }
    let _ = out.push_str(remaining);
}
