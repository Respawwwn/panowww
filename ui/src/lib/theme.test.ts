import { describe, expect, it } from "vitest";
import { applyAppearance } from "./theme";

describe("applyAppearance", () => {
  it("forces light or dark and lets auto follow the OS", () => {
    const root = document.createElement("div");
    applyAppearance("dark", root);
    expect(root.getAttribute("data-theme")).toBe("dark");
    applyAppearance("light", root);
    expect(root.getAttribute("data-theme")).toBe("light");
    applyAppearance("auto", root);
    expect(root.hasAttribute("data-theme")).toBe(false);
  });
});
