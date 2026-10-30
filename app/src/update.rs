//! Passive update check at launch (spec §14): the GitHub Releases API,
//! compared with the app version; silent on any failure.

use std::time::Duration;

use serde::Deserialize;

use crate::dto::UpdateInfo;

/// The latest release of the repository.
pub const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/Respawwwn/panowww/releases/latest";

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    html_url: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
}

/// `Some` when the latest published release is newer than `current`.
pub fn check(current: &str) -> Option<UpdateInfo> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(Duration::from_secs(8)))
        .build()
        .new_agent();
    let release: Release = agent
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", "Panowww")
        .header("Accept", "application/vnd.github+json")
        .call()
        .ok()?
        .body_mut()
        .read_json()
        .ok()?;
    newer(current, &release)
}

fn newer(current: &str, release: &Release) -> Option<UpdateInfo> {
    if release.draft || release.prerelease {
        return None;
    }
    let latest = semver::Version::parse(release.tag_name.trim_start_matches('v')).ok()?;
    let current = semver::Version::parse(current).ok()?;
    (latest > current).then(|| UpdateInfo {
        version: latest.to_string(),
        url: release.html_url.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(tag: &str) -> Release {
        Release {
            tag_name: tag.to_owned(),
            html_url: "https://github.com/Respawwwn/panowww/releases".to_owned(),
            draft: false,
            prerelease: false,
        }
    }

    #[test]
    fn only_a_newer_published_release_counts() {
        assert_eq!(
            newer("0.1.0", &release("v0.2.0")).map(|u| u.version),
            Some("0.2.0".to_owned())
        );
        assert_eq!(newer("0.2.0", &release("v0.2.0")), None);
        assert_eq!(newer("1.0.0", &release("v0.9.9")), None);
        assert_eq!(newer("0.1.0", &release("nonsense")), None);
        let mut draft = release("v9.0.0");
        draft.draft = true;
        assert_eq!(newer("0.1.0", &draft), None);
    }
}
