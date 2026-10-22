import { describe, expect, it } from "vitest";
import { createSpaceSchema, joinSpaceSchema, unlockSchema } from "../schemas/sync-secret.schema";

describe("phrase secrète d'un nouvel espace", () => {
  it("exige douze caractères et une confirmation identique", () => {
    expect(createSpaceSchema.safeParse({ passphrase: "court", confirmation: "court" }).success).toBe(false);
    expect(
      createSpaceSchema.safeParse({ passphrase: "une phrase longue", confirmation: "une phrase longe" }).success,
    ).toBe(false);
    expect(
      createSpaceSchema.safeParse({ passphrase: "une phrase longue", confirmation: "une phrase longue" }).success,
    ).toBe(true);
  });
});

describe("rejoindre un espace", () => {
  it("n'accepte jamais de scénario implicite", () => {
    const result = joinSpaceSchema.safeParse({ method: "passphrase", secret: "une phrase longue" });
    expect(result.success).toBe(false);
  });

  it("vérifie la longueur d'une clé de récupération, tirets et espaces ignorés", () => {
    const key = "ABCD-".repeat(13).slice(0, -1);
    expect(joinSpaceSchema.safeParse({ choice: "join_merge", method: "recovery", secret: key }).success).toBe(true);
    expect(joinSpaceSchema.safeParse({ choice: "join_merge", method: "recovery", secret: "ABCD-EFGH" }).success).toBe(false);
  });

  it("déverrouille avec une phrase secrète non vide", () => {
    expect(unlockSchema.safeParse({ method: "passphrase", secret: "  " }).success).toBe(false);
    expect(unlockSchema.safeParse({ method: "passphrase", secret: "ma phrase" }).success).toBe(true);
  });
});
