import { describe, expect, it } from "vitest";
import { idProvider, versProvider } from "../providers";

describe("fournisseurs IA", () => {
  it("reconnaît le variant personnalisé sérialisé par serde", () => {
    expect(idProvider({ custom: "maison" })).toBe("custom");
    expect(idProvider("openai")).toBe("openai");
    expect(versProvider("custom")).toEqual({ custom: "custom" });
  });
});
