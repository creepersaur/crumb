use once_cell::sync::Lazy;
use regex::Regex;

// Matches trailing metadata chunks like:
// (x64 1.0.5-x64 Windows), (User), (en-US), v1.2.3, 1.0.5-x64, [x64], etc.
static TRAILING_PAREN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"[\(\[][^()\[\]]*[\)\]]\s*$").unwrap()
});

static VERSION_TOKEN: Lazy<Regex> = Lazy::new(|| {
    // v1.2.3 / 1.2.3.4 / 1.0.5-x64 etc, as a trailing standalone token
    Regex::new(r"(?i)\s+v?\d+(\.\d+){1,3}([-_][a-z0-9]+)?\s*$").unwrap()
});

static ARCH_LOCALE_TOKEN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\s+(x64|x86|arm64|amd64|win64|win32|en-us|en-gb|user|system)\s*$").unwrap()
});

fn paren_is_metadata(inner: &str) -> bool {
    let inner = inner.trim();
    let lower = inner.to_lowercase();

    // Pure version-ish content
    let arch_locale = ["x64", "x86", "arm64", "amd64", "win64", "win32", "en-us", "en-gb", "user", "system"];
    let tokens: Vec<&str> = lower.split_whitespace().collect();

    if tokens.is_empty() {
        return true;
    }

    tokens.iter().all(|t| {
        let t = t.trim_matches(|c: char| !c.is_alphanumeric());
        arch_locale.contains(&t)
            || t.chars().next().map_or(false, |c| c.is_ascii_digit())
            || t == "v" // stray version prefix
            || VERSION_TOKEN.is_match(&format!(" {t}"))
    })
}

pub(crate) fn clean_app_name(name: &str) -> String {
    let mut result = name.trim().to_string();

    loop {
        let before = result.clone();

        // Strip trailing (...) or [...] if it looks like metadata
        if let Some(m) = TRAILING_PAREN.find(&result) {
            let inner = &result[m.start() + 1..m.end() - 1];
            if paren_is_metadata(inner) {
                result.truncate(m.start());
                result = result.trim_end().to_string();
            }
        }

        // Strip trailing bare version tokens (e.g. "MyApp 1.0.5-x64")
        result = VERSION_TOKEN.replace(&result, "").trim_end().to_string();

        // Strip trailing bare arch/locale tokens not in parens
        result = ARCH_LOCALE_TOKEN.replace(&result, "").trim_end().to_string();

        // Strip trailing hyphens/dashes left dangling after removals
        result = result
            .trim_end_matches(|c: char| c == '-' || c.is_whitespace())
            .to_string();

        if result == before {
            break;
        }
    }

    result.trim().to_string()
}