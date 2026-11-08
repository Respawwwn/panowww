//! Passive update check at launch (spec §14): the GitHub Releases API,
//! compared with the app version; silent on any failure.
//!
//! Reads the releases list rather than `releases/latest` so that the
//! *Include pre-releases* switch can see them: GitHub leaves pre-releases
//! out of `latest`. With the switch off only releases not marked as
//! pre-release count, whatever their version.

use std::time::Duration;

use serde::Deserialize;

use crate::dto::UpdateInfo;

/// The most recent releases of the repository, newest first.
pub const RELEASES_URL: &str =
    "https://api.github.com/repos/Respawwwn/panowww/releases?per_page=20";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// `Some` when a published release is newer than `current`.
pub fn check(current: &str, include_prereleases: bool) -> Option<UpdateInfo> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(8)))
        .build()
        .new_agent();
    let releases: Vec<Release> = agent
        .get(RELEASES_URL)
        .header("User-Agent", "Panowww")
        .header("Accept", "application/vnd.github+json")
        .call()
        .ok()?
        .body_mut()
        .read_json()
        .ok()?;
    newer(current, include_prereleases, &releases)
}

fn newer(current: &str, include_prereleases: bool, releases: &[Release]) -> Option<UpdateInfo> {
    let current = semver::Version::parse(current).ok()?;
    releases
        .iter()
        .filter(|release| !release.draft && (include_prereleases || !release.prerelease))
        .filter_map(|release| {
            let version = semver::Version::parse(release.tag_name.trim_start_matches('v')).ok()?;
            Some((version, release))
        })
        .filter(|(version, _)| *version > current)
        .max_by(|(a, _), (b, _)| a.cmp(b))
        .map(|(version, release)| UpdateInfo {
            version: version.to_string(),
            url: release.html_url.clone(),
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str, prerelease: bool) -> Release {
        Release {
            tag_name: tag.to_owned(),
            html_url: format!("https://github.com/Respawwwn/panowww/releases/tag/{tag}"),
            draft: false,
            prerelease,
        }
    }

    fn version(current: &str, include_prereleases: bool, releases: &[Release]) -> Option<String> {
        newer(current, include_prereleases, releases).map(|u| u.version)
    }

    #[test]
    fn a_newer_published_release_counts_and_links_its_page() {
        let releases = [release("v0.2.0", false), release("v0.1.0", false)];
        let update = newer("0.1.0", false, &releases).expect("newer");
        assert_eq!(update.version, "0.2.0");
        assert_eq!(
            update.url,
            "https://github.com/Respawwwn/panowww/releases/tag/v0.2.0"
        );
    }

    #[test]
    fn the_same_or_an_older_release_does_not_count() {
        assert_eq!(version("0.2.0", false, &[release("v0.2.0", false)]), None);
        assert_eq!(version("1.0.0", false, &[release("v0.9.9", false)]), None);
        assert_eq!(version("0.1.0", false, &[release("nonsense", false)]), None);
        assert_eq!(version("0.1.0", false, &[]), None);
    }

    #[test]
    fn the_highest_version_wins_whatever_the_order() {
        let releases = [
            release("v0.2.0", false),
            release("v0.4.0", false),
            release("v0.3.0", false),
        ];
        assert_eq!(version("0.1.0", false, &releases), Some("0.4.0".to_owned()));
    }

    #[test]
    fn a_draft_never_counts() {
        let mut draft = release("v9.0.0", false);
        draft.draft = true;
        assert_eq!(version("0.1.0", true, &[draft]), None);
    }

    #[test]
    fn with_the_switch_off_a_pre_release_never_counts() {
        let releases = [release("v1.1.0-beta.1", true), release("v1.0.1", false)];
        assert_eq!(version("1.0.0", false, &releases), Some("1.0.1".to_owned()));
        assert_eq!(version("1.0.1", false, &releases), None);
        assert_eq!(version("0.1.0", false, &[release("v0.2.0", true)]), None);
    }

    #[test]
    fn with_the_switch_on_pre_releases_count_too() {
        let releases = [release("v1.1.0-beta.1", true), release("v1.0.1", false)];
        assert_eq!(
            version("1.0.1", true, &releases),
            Some("1.1.0-beta.1".to_owned())
        );
        assert_eq!(
            version("0.1.0", true, &[release("v0.2.0", true)]),
            Some("0.2.0".to_owned())
        );
        assert_eq!(
            version("1.0.0-beta.2", true, &[release("v1.0.0", false)]),
            Some("1.0.0".to_owned())
        );
    }
}
