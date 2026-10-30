//! Watches the Games folder so a Game edited by hand, or a damaged file
//! repaired in an editor, is re-read (spec §5). The callback runs on the
//! watcher's thread; the caller re-lists the folder.

use std::path::{Path, PathBuf};

use notify::{EventKind, RecommendedWatcher, RecursiveMode, Watcher as _};
use panowww_core::game::name::FILE_SUFFIX;

use crate::StoreError;

/// A `.panowww.json` file of the Games folder changed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameFileEvent {
    /// Created or modified: read it again.
    Changed(PathBuf),
    Removed(PathBuf),
}

/// Keeps the watch alive; dropping it stops the callbacks.
pub struct GamesWatcher {
    _watcher: RecommendedWatcher,
}

impl std::fmt::Debug for GamesWatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("GamesWatcher")
    }
}

/// Starts watching `dir` and calls `on_event` for every Game file that
/// changes. Temporary files of atomic writes and other files are ignored.
pub fn watch(
    dir: &Path,
    on_event: impl Fn(GameFileEvent) + Send + 'static,
) -> Result<GamesWatcher, StoreError> {
    let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
        let Ok(event) = result else {
            return;
        };
        let kind = match event.kind {
            EventKind::Create(_) | EventKind::Modify(_) => Some(true),
            EventKind::Remove(_) => Some(false),
            _ => None,
        };
        let Some(changed) = kind else {
            return;
        };
        for path in event.paths.into_iter().filter(|p| is_game_file(p)) {
            // A rename lands as Modify on the destination, Remove on the
            // source. Whatever notify says, a missing file was removed.
            if changed && path.exists() {
                on_event(GameFileEvent::Changed(path));
            } else {
                on_event(GameFileEvent::Removed(path));
            }
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok(GamesWatcher { _watcher: watcher })
}

fn is_game_file(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .is_some_and(|n| n.ends_with(FILE_SUFFIX) && !n.starts_with('.'))
}

#[cfg(test)]
mod tests {
    use std::sync::mpsc;
    use std::time::Duration;

    use super::*;

    const PATIENCE: Duration = Duration::from_secs(10);

    #[test]
    fn game_files_are_recognised() {
        assert!(is_game_file(Path::new("/g/Elden Ring.panowww.json")));
        assert!(!is_game_file(Path::new("/g/.Elden Ring.panowww.json.tmp")));
        assert!(!is_game_file(Path::new("/g/notes.txt")));
        assert!(!is_game_file(Path::new("/g/.hidden.panowww.json")));
    }

    #[test]
    fn a_written_then_deleted_game_file_is_reported() {
        let dir = tempfile::tempdir().unwrap();
        // macOS reports events under the real path (/private/var), not the
        // symlinked one tempfile hands out.
        let root = dir.path().canonicalize().unwrap();
        let (tx, rx) = mpsc::channel();
        let _watcher = watch(&root, move |event| {
            let _ = tx.send(event);
        })
        .unwrap();

        let path = root.join("Test.panowww.json");
        std::fs::write(&path, "{}").unwrap();
        std::fs::write(root.join("ignored.txt"), "x").unwrap();
        let event = rx.recv_timeout(PATIENCE).expect("a change event");
        assert_eq!(event, GameFileEvent::Changed(path.clone()));

        std::fs::remove_file(&path).unwrap();
        let deadline = std::time::Instant::now() + PATIENCE;
        loop {
            let event = rx
                .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
                .expect("a removal event");
            if event == GameFileEvent::Removed(path.clone()) {
                break;
            }
        }
    }
}
