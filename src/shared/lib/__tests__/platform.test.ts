import { describe, expect, it } from "vitest";
import { formatShortcut } from "../platform";

describe("notation des raccourcis", () => {
  it("colle les glyphes sous macOS", () => {
    expect(formatShortcut("mod+k", "mac")).toBe("⌘K");
    expect(formatShortcut("mod+enter", "mac")).toBe("⌘⏎");
    expect(formatShortcut("mod+backspace", "mac")).toBe("⌘⌫");
    expect(formatShortcut("shift+enter", "mac")).toBe("⇧⏎");
  });

  it("écrit Ctrl en entier et sépare par une espace ailleurs, sans +", () => {
    expect(formatShortcut("mod+k", "other")).toBe("Ctrl K");
    expect(formatShortcut("mod+,", "other")).toBe("Ctrl ,");
    expect(formatShortcut("mod+backspace", "other")).toBe("Ctrl Suppr");
    expect(formatShortcut("shift+enter", "other")).toBe("Maj ⏎");
  });

  it("garde une touche seule telle quelle, en capitale", () => {
    expect(formatShortcut("n", "mac")).toBe("N");
    expect(formatShortcut("n", "other")).toBe("N");
    expect(formatShortcut("escape", "other")).toBe("Échap");
  });
});
