import { describe, expect, it } from "vitest";
import { identifiantProvider, versProvider } from "../providers";

describe("fournisseurs IA", () => {
  it("reconnaît le variant personnalisé sérialisé par serde", () => {
    expect(identifiantProvider({ custom: "maison" })).toBe("custom");
    expect(identifiantProvider("open_a_i")).toBe("open_a_i");
    expect(versProvider("custom")).toEqual({ custom: "custom" });
  });
});
