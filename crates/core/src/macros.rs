//! Crate-wide declarative macros for the kinds of enum→string boilerplate
//! that proliferated across the port. Each macro is narrow and does one
//! thing; see the individual doc comments for which to reach for.

/// Generate `impl $Enum { pub fn $method(self) -> &'static str }` from a
/// `Variant => "string"` table. The body is just one `match self` arm per
/// variant — use this any time the only thing that varies is the returned
/// constant string.
///
/// Works equally for stable English identifiers (save keys, log tags),
/// snake_case ASCII names, and translated `t!(...)` labels — the table
/// values are arbitrary `&'static str` expressions.
///
/// ```ignore
/// enum_str_method! {
///     BehaviorId::name;
///     Idle => "idle",
///     Sleeping => "sleeping",
///     // ...
/// }
/// ```
#[macro_export]
macro_rules! enum_str_method {
    (
        $(#[$meta:meta])*
        $Enum:ident :: $method:ident;
        $($Variant:ident => $value:expr),+ $(,)?
    ) => {
        impl $Enum {
            $(#[$meta])*
            pub fn $method(self) -> &'static str {
                match self {
                    $($Enum::$Variant => $value,)+
                }
            }
        }
    };
}

/// Generate a paired `pub fn $to(_: $Enum) -> &'static str` /
/// `pub fn $from(_: &str) -> Option<$Enum>` from a `Variant => "key"` table.
///
/// Use for save-file fields where an unknown serialized key should bubble
/// up as a load failure rather than silently being replaced with a sentinel.
/// (Matches the Python convention for these enums.)
#[macro_export]
macro_rules! enum_key_pair_option {
    (
        $to:ident, $from:ident, $Enum:ident;
        $($Variant:ident => $key:literal),+ $(,)?
    ) => {
        pub fn $to(e: $Enum) -> &'static str {
            match e {
                $($Enum::$Variant => $key,)+
            }
        }
        pub fn $from(s: &str) -> Option<$Enum> {
            Some(match s {
                $($key => $Enum::$Variant,)+
                _ => return None,
            })
        }
    };
}

/// Generate a paired `pub fn $to(_: $Enum) -> &'static str` /
/// `pub fn $from(_: &str) -> $Enum` (returns the variant directly, with a
/// designated `default:` for unknown input). Supports per-variant aliases:
/// the first listed key is the canonical one used by `$to`; additional
/// `| "alias"` patterns are accepted by `$from` for legacy-save tolerance.
///
/// ```ignore
/// enum_key_pair_default! {
///     layer_save_key, layer_from_key, PlantLayer;
///     default: PlantLayer::Midground;
///     Background => "background" | "bg",
///     Midground  => "midground",
///     Foreground => "foreground" | "fg",
/// }
/// ```
#[macro_export]
macro_rules! enum_key_pair_default {
    (
        $to:ident, $from:ident, $Enum:ident;
        default: $default:expr;
        $($Variant:ident => $canonical:literal $(| $alias:literal)*),+ $(,)?
    ) => {
        pub fn $to(e: $Enum) -> &'static str {
            match e {
                $($Enum::$Variant => $canonical,)+
            }
        }
        pub fn $from(s: &str) -> $Enum {
            match s {
                $($canonical $(| $alias)* => $Enum::$Variant,)+
                _ => $default,
            }
        }
    };
}
