import { describe, expect, it } from "vitest";
import { searxngUrlSchema } from "../web-search.schema";

describe("instance SearXNG", () => {
  it("accepte une adresse HTTP(S) ou le vide", () => {
    expect(searxngUrlSchema.parse("https://searx.exemple.fr")).toBe("https://searx.exemple.fr");
    expect(searxngUrlSchema.parse("  ")).toBeNull();
  });

  it("refuse tout autre schéma et une adresse démesurée", () => {
    expect(searxngUrlSchema.safeParse("javascript:alert(1)").success).toBe(false);
    expect(searxngUrlSchema.safeParse(`https://${"a".repeat(200)}.fr`).success).toBe(false);
  });
});
