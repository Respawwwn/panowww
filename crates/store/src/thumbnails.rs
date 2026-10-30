//! History thumbnails (spec §9): a small JPEG per Shot, generated on demand
//! into `cache/thumbnails/<session-id>/rNN_cNN.jpg` from the PNG, never
//! into the Session folder.

use std::path::{Path, PathBuf};

use panowww_core::runner::thumbnail::thumbnail;

use crate::StoreError;
use crate::paths::AppPaths;
use crate::png;
use crate::session::SessionRecord;

/// Where the thumbnail of `shot_file` (`rNN_cNN.png`) of `session` lives.
pub fn thumbnail_path(paths: &AppPaths, session: &SessionRecord, shot_file: &str) -> PathBuf {
    let stem = Path::new(shot_file)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned();
    paths
        .thumbnails_dir()
        .join(session.id.to_string())
        .join(format!("{stem}.jpg"))
}

/// The thumbnail of one Shot, generated from `<folder>/<shot_file>` if it
/// is not cached yet.
pub fn history_thumbnail(
    paths: &AppPaths,
    folder: &Path,
    session: &SessionRecord,
    shot_file: &str,
) -> Result<PathBuf, StoreError> {
    let target = thumbnail_path(paths, session, shot_file);
    if target.is_file() {
        return Ok(target);
    }
    let frame = png::read(&folder.join(shot_file))?;
    let jpeg = thumbnail(&frame).ok_or_else(|| StoreError::Png {
        path: folder.join(shot_file),
        message: "the Shot could not be reduced to a thumbnail".to_owned(),
    })?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|e| StoreError::io("create", parent, e))?;
    }
    std::fs::write(&target, jpeg.0).map_err(|e| StoreError::io("write", &target, e))?;
    Ok(target)
}

/// Forgets the cached thumbnails of a deleted Session.
pub fn remove_thumbnails(paths: &AppPaths, session: &SessionRecord) -> Result<(), StoreError> {
    let dir = paths.thumbnails_dir().join(session.id.to_string());
    match std::fs::remove_dir_all(&dir) {
        Ok(()) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(StoreError::io("delete", &dir, e)),
    }
}
