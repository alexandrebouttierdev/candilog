import { afterEach, describe, expect, it, vi } from "vitest";
import {
  completionSoundEnabled,
  playCompletionSound,
  setCompletionSoundEnabled,
} from "../completion-sound";

/**
 * Simule le contexte audio du moteur, sans jouer de son réel.
 *
 * jsdom n'implémente pas `AudioContext` : ce double reproduit uniquement ce dont
 * `playCompletionSound` a besoin, et démarre `suspended` — l'état que la politique de
 * lecture automatique impose quand le contexte est créé hors d'un geste utilisateur, ce
 * qui est toujours le cas ici puisque l'appel arrive après un `await`.
 */
class FakeAudioContext {
  state: "suspended" | "running" = "suspended";
  currentTime = 0;
  destination = {};
  resume = vi.fn(() => {
    this.state = "running";
    return Promise.resolve();
  });
  close = vi.fn(() => Promise.resolve());
  createOscillator() {
    return {
      type: "sine",
      frequency: { setValueAtTime: vi.fn() },
      connect: vi.fn(),
      start: vi.fn(),
      stop: vi.fn(),
    };
  }
  createGain() {
    return {
      gain: { setValueAtTime: vi.fn(), exponentialRampToValueAtTime: vi.fn() },
      connect: vi.fn(),
    };
  }
}

describe("son de fin de traitement", () => {
  afterEach(() => {
    window.localStorage.clear();
    vi.unstubAllGlobals();
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

  it("réveille un contexte audio suspendu, sinon le son reste inaudible", () => {
    const context = new FakeAudioContext();
    vi.stubGlobal(
      "AudioContext",
      vi.fn(function AudioContext() {
        return context;
      }),
    );

    playCompletionSound();

    expect(context.resume).toHaveBeenCalledOnce();
  });

  it("ne tente pas de reprise quand le contexte démarre déjà actif", () => {
    const context = new FakeAudioContext();
    context.state = "running";
    vi.stubGlobal(
      "AudioContext",
      vi.fn(function AudioContext() {
        return context;
      }),
    );

    playCompletionSound();

    expect(context.resume).not.toHaveBeenCalled();
  });
});
