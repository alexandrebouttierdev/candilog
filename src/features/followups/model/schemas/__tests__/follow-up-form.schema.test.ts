import { describe, expect, it } from "vitest";
import { followUpFormSchema } from "../follow-up-form.schema";

const BASE = {
  application_id: "11111111-1111-1111-1111-111111111111",
  follow_up_date: "27-08-2026",
  type: "Email",
  notes: "",
};

describe("schéma du formulaire relance", () => {
  it("transmet la date au format attendu par la base", () => {
    expect(followUpFormSchema.parse(BASE).follow_up_date).toBe("2026-08-27");
  });

  it("refuse une date inexistante", () => {
    const resultat = followUpFormSchema.safeParse({ ...BASE, follow_up_date: "31-02-2026" });
    expect(resultat.success).toBe(false);
    if (!resultat.success) {
      expect(resultat.error.issues[0]?.message).toContain("JJ-MM-AAAA");
    }
  });
});
