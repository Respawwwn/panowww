import { expect, test } from "vitest";
import { formatBytes, formatDuration, shotLabel } from "./format";

test("durations match the Rust format", () => {
  expect(formatDuration(0)).toBe("≈ 0 s");
  expect(formatDuration(45_000)).toBe("≈ 45 s");
  expect(formatDuration(60_000)).toBe("≈ 1 min");
  expect(formatDuration(260_000)).toBe("≈ 4 min 20 s");
  expect(formatDuration(259_600)).toBe("≈ 4 min 20 s");
  expect(formatDuration(3_600_000)).toBe("≈ 1 h");
  expect(formatDuration(4_320_000)).toBe("≈ 1 h 12 min");
});

test("bytes pick a unit", () => {
  expect(formatBytes(900)).toBe("900 B");
  expect(formatBytes(12_300)).toBe("12 KB");
  expect(formatBytes(480_000_000)).toBe("480 MB");
  expect(formatBytes(1_250_000_000)).toBe("1.3 GB");
});

test("shot labels are two-digit", () => {
  expect(shotLabel(1, 1)).toBe("r01_c01");
  expect(shotLabel(6, 13)).toBe("r06_c13");
});
