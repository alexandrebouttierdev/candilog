import { afterEach, describe, expect, it } from "vitest";
import {
  completionSoundEnabled,
  playCompletionSound,
  setCompletionSoundEnabled,
} from "../completion-sound";

describe("son de fin de traitement", () => {
  afterEach(() => {
    window.localStorage.clear();
  });

  it("est actif tant que rien n'a été choisi", () => {
    expect(completionSoundEnabled()).toBe(true);
  });

  it("retient la désactivation puis la réactivation", () => {
    setCompletionSoundEnabled(false);
    expect(completionSoundEnabled()).toBe(false);

    setCompletionSoundEnabled(true);
    expect(completionSoundEnabled()).toBe(true);
  });

  it("reste silencieux sans sortie audio au lieu d'échouer", () => {
    expect(() => playCompletionSound()).not.toThrow();
  });
});
