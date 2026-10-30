//! Game names and file names unique on one machine (spec §5). Names compare
//! case-insensitively after trimming; file names are `<name>.panowww.json`
//! with forbidden characters replaced and ` (2)` on collision.

use super::validate::NAME_MAX_CHARS;

/// Every Game file ends with this.
pub const FILE_SUFFIX: &str = ".panowww.json";

/// Stem used when a name leaves no usable character for a file name.
const EMPTY_STEM: &str = "Game";

/// Characters no file system here accepts in a file name.
const FORBIDDEN: &[char] = &['/', '\\', ':', '*', '?', '"', '<', '>', '|'];

/// Stems Windows refuses whatever the extension.
const RESERVED_STEMS: &[&str] = &[
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Whether `name` clashes with one of `others`, ignoring case and padding.
pub fn is_taken<S: AsRef<str>>(name: &str, others: impl IntoIterator<Item = S>) -> bool {
    let wanted = fold(name);
    others
        .into_iter()
        .any(|other| fold(other.as_ref()) == wanted)
}

/// `desired`, or `desired (2)`, `desired (3)`… the first not taken by
/// `others`. Stays within [`NAME_MAX_CHARS`] by shortening the base.
pub fn unique<S: AsRef<str>>(desired: &str, others: &[S]) -> String {
    let desired = desired.trim();
    if !is_taken(desired, others) {
        return fit(desired, "");
    }
    first_free(|n| fit(desired, &format!(" ({n})")), others)
}

/// `candidate(2)`, `candidate(3)`… the first not taken by `others`.
fn first_free<S: AsRef<str>>(candidate: impl Fn(u32) -> String, others: &[S]) -> String {
    let mut n = 2u32;
    loop {
        let name = candidate(n);
        if n == u32::MAX || !is_taken(&name, others) {
            return name;
        }
        n += 1;
    }
}

/// The name of a Duplicate: `<name> copy`, made unique among `others`.
pub fn copy_name<S: AsRef<str>>(name: &str, others: &[S]) -> String {
    unique(&fit(name.trim(), " copy"), others)
}

/// `<stem>.panowww.json` for `name`, with forbidden characters replaced by
/// `-` and ` (2)`, ` (3)`… when `taken_files` already holds it (compared
/// ignoring case, as macOS and Windows file systems do).
pub fn file_name<S: AsRef<str>>(name: &str, taken_files: &[S]) -> String {
    let stem = sanitize_stem(name);
    let candidate = format!("{stem}{FILE_SUFFIX}");
    if !is_taken(&candidate, taken_files) {
        return candidate;
    }
    first_free(|n| format!("{stem} ({n}){FILE_SUFFIX}"), taken_files)
}

/// `name` made safe as a file stem on macOS and Windows.
pub fn sanitize_stem(name: &str) -> String {
    let replaced: String = name
        .chars()
        .map(|c| {
            if FORBIDDEN.contains(&c) || c.is_control() {
                '-'
            } else {
                c
            }
        })
        .collect();
    let trimmed = replaced.trim().trim_end_matches('.').trim_end();
    if trimmed.is_empty() {
        return EMPTY_STEM.to_owned();
    }
    if RESERVED_STEMS
        .iter()
        .any(|r| r.eq_ignore_ascii_case(trimmed))
    {
        return format!("{trimmed}-");
    }
    trimmed.to_owned()
}

/// `base` + `suffix`, `base` shortened so the whole fits [`NAME_MAX_CHARS`].
fn fit(base: &str, suffix: &str) -> String {
    let room = NAME_MAX_CHARS.saturating_sub(suffix.chars().count());
    let mut out: String = base.chars().take(room).collect();
    let trimmed_len = out.trim_end().len();
    out.truncate(trimmed_len);
    out.push_str(suffix);
    out
}

fn fold(name: &str) -> String {
    name.trim().to_lowercase()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taken_ignores_case_and_padding() {
        assert!(is_taken("elden ring", ["Elden Ring"]));
        assert!(is_taken(" Elden Ring ", ["elden ring"]));
        assert!(is_taken("ÉLAN", ["élan"]));
        assert!(!is_taken("Elden Ring", ["Elden Ring 2"]));
        assert!(!is_taken("x", Vec::<String>::new()));
    }

    #[test]
    fn unique_appends_a_counter_only_on_clash() {
        assert_eq!(unique("Elden Ring", &["Other"]), "Elden Ring");
        assert_eq!(unique("Elden Ring", &["elden ring"]), "Elden Ring (2)");
        assert_eq!(
            unique("Elden Ring", &["Elden Ring", "Elden Ring (2)"]),
            "Elden Ring (3)"
        );
        assert_eq!(unique("  padded  ", &["x"]), "padded");
    }

    #[test]
    fn unique_keeps_the_name_within_sixty_characters() {
        let long = "a".repeat(60);
        let name = unique(&long, &[long.as_str()]);
        assert_eq!(name.chars().count(), 60);
        assert!(name.ends_with(" (2)"));
    }

    #[test]
    fn copy_name_adds_copy_then_counts() {
        assert_eq!(copy_name("Elden Ring", &["Elden Ring"]), "Elden Ring copy");
        assert_eq!(
            copy_name("Elden Ring", &["Elden Ring", "Elden Ring copy"]),
            "Elden Ring copy (2)"
        );
        assert_eq!(copy_name(&"b".repeat(60), &["x"]).chars().count(), 60);
    }

    #[test]
    fn file_name_replaces_forbidden_characters() {
        assert_eq!(
            file_name("Elden Ring – photo mode", &[""; 0]),
            "Elden Ring – photo mode.panowww.json"
        );
        assert_eq!(
            file_name("a/b\\c:d*e?f\"g<h>i|j", &[""; 0]),
            "a-b-c-d-e-f-g-h-i-j.panowww.json"
        );
        assert_eq!(file_name("tab\there", &[""; 0]), "tab-here.panowww.json");
    }

    #[test]
    fn file_name_handles_windows_edge_cases() {
        assert_eq!(sanitize_stem("ends with dots..."), "ends with dots");
        assert_eq!(sanitize_stem("  spaced  "), "spaced");
        assert_eq!(sanitize_stem("con"), "con-");
        assert_eq!(sanitize_stem("LPT1"), "LPT1-");
        assert_eq!(sanitize_stem("???"), "---");
        assert_eq!(sanitize_stem("..."), "Game");
        assert_eq!(sanitize_stem(""), "Game");
    }

    #[test]
    fn file_name_counts_on_collision_ignoring_case() {
        let taken = ["elden ring.panowww.json", "Elden Ring (2).panowww.json"];
        assert_eq!(
            file_name("Elden Ring", &taken),
            "Elden Ring (3).panowww.json"
        );
        assert_eq!(file_name("Elden Ring", &[""; 0]), "Elden Ring.panowww.json");
    }
}
