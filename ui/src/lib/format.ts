// Human-readable numbers for the screens: durations as the Rust side
// writes them (`≈ 4 min 20 s`), bytes, dates.

/** `≈ 4 min 20 s`, `≈ 45 s`, `≈ 1 h 12 min`: rounded to the second, zero parts dropped. */
export function formatDuration(ms: number): string {
  const secs = Math.round(Math.max(0, ms) / 1000);
  const h = Math.floor(secs / 3600);
  const m = Math.floor((secs % 3600) / 60);
  const s = secs % 60;
  if (h > 0) {
    return m > 0 ? `≈ ${h} h ${m} min` : `≈ ${h} h`;
  }
  if (m > 0) {
    return s > 0 ? `≈ ${m} min ${s} s` : `≈ ${m} min`;
  }
  return `≈ ${s} s`;
}

/** `1.2 GB`, `480 MB`, `12 KB`. */
export function formatBytes(bytes: number): string {
  if (bytes >= 1e9) {
    return `${(bytes / 1e9).toFixed(1)} GB`;
  }
  if (bytes >= 1e6) {
    return `${Math.round(bytes / 1e6)} MB`;
  }
  if (bytes >= 1e3) {
    return `${Math.round(bytes / 1e3)} KB`;
  }
  return `${bytes} B`;
}

/** `30 Oct 2026, 15:04` in the user's locale. */
export function formatDate(at: Date | string | number): string {
  const date = at instanceof Date ? at : new Date(at);
  if (Number.isNaN(date.getTime())) {
    return "";
  }
  return date.toLocaleString(undefined, {
    day: "numeric",
    month: "short",
    year: "numeric",
    hour: "2-digit",
    minute: "2-digit",
  });
}

/** `rNN_cNN` for a thumbnail label. */
export function shotLabel(row: number, col: number): string {
  return `r${String(row).padStart(2, "0")}_c${String(col).padStart(2, "0")}`;
}
