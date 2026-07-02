use std::collections::BTreeMap;

/// A resolved set of color tokens, keyed by the same `--token-name` strings the web app
/// used as CSS custom properties (e.g. `--bg`, `--heading-1`). Kept as a string-keyed map
/// (not a fixed struct) so config's free-form per-theme overrides
/// (`UiConfig::colors: BTreeMap<String, BTreeMap<String, String>>`) can merge over it by
/// name without `core` needing to know about every override key in advance. Port of
/// `web/src/lib/theme.ts`'s `Tokens`.
pub type Tokens = BTreeMap<String, String>;

fn tokens(pairs: &[(&str, &str)]) -> Tokens {
    pairs
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect()
}

/// The built-in light palette. Port of `web/src/lib/theme.ts`'s `LIGHT`.
pub fn light() -> Tokens {
    tokens(&[
        ("--bg", "#fbfcfe"),
        ("--fg", "#1b2330"),
        ("--muted", "#5b6675"),
        ("--accent", "#2f6df6"),
        ("--heading-1", "#1d4ed8"),
        ("--heading-2", "#2563eb"),
        ("--heading-3", "#3b82f6"),
        ("--heading-4", "#60a5fa"),
        ("--heading-5", "#7dabfb"),
        ("--heading-6", "#9cc2fc"),
        ("--todo-done", "#8a93a3"),
        ("--meta", "#6b7686"),
        ("--status-bar", "#eef2f9"),
        ("--edit-line-bg", "#eaf1ff"),
        ("--edit-bar-bg", "#e2ebff"),
        ("--rule", "#d9e0ec"),
        ("--cursor", "#1b2330"),
        ("--blockquote-border", "#93c5fd"),
        ("--highlight-bg", "#fef08a"),
    ])
}

/// The built-in dark palette. Port of `web/src/lib/theme.ts`'s `DARK`. `web/`'s in-code
/// `DARK` object omits `--todo-done` (only `LIGHT` and `web/src/app.css`'s `:root` default
/// define it, both `#8a93a3`) — the web app never overrides it for dark mode, so the
/// rendered color is always that same muted gray regardless of theme. This port makes
/// that explicit rather than relying on a CSS-cascade accident: `--todo-done` is included
/// here with the identical value.
pub fn dark() -> Tokens {
    tokens(&[
        ("--bg", "#161a26"),
        ("--fg", "#e7ecf5"),
        ("--muted", "#97a1b3"),
        ("--accent", "#2f6df6"),
        ("--heading-1", "#1d4ed8"),
        ("--heading-2", "#3b82f6"),
        ("--heading-3", "#60a5fa"),
        ("--heading-4", "#7dabfb"),
        ("--heading-5", "#9cc2fc"),
        ("--heading-6", "#9cc2fc"),
        ("--todo-done", "#8a93a3"),
        ("--meta", "#97a1b3"),
        ("--status-bar", "#1f2535"),
        ("--edit-line-bg", "#222a3d"),
        ("--edit-bar-bg", "#2a344c"),
        ("--rule", "#2d3650"),
        ("--cursor", "#e7ecf5"),
        ("--blockquote-border", "#3b82f6"),
        ("--highlight-bg", "#713f12"),
    ])
}

/// The built-in tokens for `theme` (anything other than `"dark"` is treated as light).
/// Port of `web/src/lib/theme.ts`'s `builtinTokens`.
pub fn builtin_tokens(theme: &str) -> Tokens {
    if theme == "dark" { dark() } else { light() }
}

/// The opposite of `theme` (anything not `"dark"` flips to `"dark"`). Port of
/// `web/src/lib/theme.ts`'s `nextTheme`.
pub fn next_theme(theme: &str) -> String {
    if theme == "dark" {
        "light".to_string()
    } else {
        "dark".to_string()
    }
}

/// Merge the built-in tokens for `theme` with `overrides[theme]` (config's per-theme
/// color overrides), overrides winning. Port of `web/src/lib/theme.ts`'s
/// `resolveTokens`.
pub fn resolve_tokens(theme: &str, overrides: &BTreeMap<String, Tokens>) -> Tokens {
    let mut result = builtin_tokens(theme);
    if let Some(over) = overrides.get(theme) {
        for (k, v) in over {
            result.insert(k.clone(), v.clone());
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_built_in_light_tokens_by_default() {
        assert_eq!(
            resolve_tokens("light", &BTreeMap::new())["--bg"],
            light()["--bg"]
        );
    }

    #[test]
    fn returns_dark_tokens_for_the_dark_theme() {
        assert_eq!(
            resolve_tokens("dark", &BTreeMap::new())["--bg"],
            dark()["--bg"]
        );
    }

    #[test]
    fn falls_back_to_light_for_unknown_themes() {
        assert_eq!(
            resolve_tokens("neon", &BTreeMap::new())["--bg"],
            light()["--bg"]
        );
    }

    #[test]
    fn applies_per_theme_config_overrides_over_the_base() {
        let mut overrides = BTreeMap::new();
        overrides.insert("dark".to_string(), tokens(&[("--bg", "#000000")]));
        let t = resolve_tokens("dark", &overrides);
        assert_eq!(t["--bg"], "#000000");
        assert_eq!(t["--fg"], dark()["--fg"]);
    }

    #[test]
    fn defines_the_rule_and_edit_bar_tokens_for_both_themes() {
        for t in [light(), dark()] {
            assert!(t["--rule"].starts_with('#'));
            assert!(t["--edit-bar-bg"].starts_with('#'));
        }
    }

    #[test]
    fn next_theme_flips_dark_to_light_and_anything_else_to_dark() {
        assert_eq!(next_theme("dark"), "light");
        assert_eq!(next_theme("light"), "dark");
        assert_eq!(next_theme("whatever"), "dark");
    }
}
