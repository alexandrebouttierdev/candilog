import { afterEach, describe, expect, it } from "vitest";
import { markOnboardingCompleted, onboardingCompleted } from "../onboarding-storage";

describe("état du tour d'accueil", () => {
  afterEach(() => {
    window.localStorage.clear();
  });

  it("n'est pas vu tant que rien n'a été enregistré", () => {
    expect(onboardingCompleted()).toBe(false);
  });

  it("reste vu après avoir été marqué comme terminé", () => {
    markOnboardingCompleted();
    expect(onboardingCompleted()).toBe(true);
  });
});
