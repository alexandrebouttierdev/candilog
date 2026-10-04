import "@testing-library/jest-dom/vitest";
import { configure } from "@testing-library/dom";

/**
 * Délai d'attente des utilitaires asynchrones (`waitFor`, `findBy*`).
 *
 * Le défaut de Testing Library est d'une seconde, mesurée en temps réel : une machine
 * chargée la dépasse sur un rendu qui, lui, est correct. Les échecs qui en résultaient
 * décrivaient un élément « introuvable » alors qu'il apparaissait juste après. Cinq
 * secondes restent très en deçà du plafond par test, donc une attente réellement bloquée
 * échoue toujours.
 */
configure({ asyncUtilTimeout: 5_000 });

/**
 * `localStorage` de l'environnement jsdom.
 *
 * Node 26 expose son propre `localStorage` global, **indéfini** tant que
 * `--localstorage-file` n'est pas passé (il le signale par un `ExperimentalWarning`). Comme
 * Vitest fait de `window` le `globalThis` du worker, cet accesseur masque celui de jsdom —
 * qui fonctionne pourtant très bien — et `window.localStorage` vaut `undefined`. Seul
 * `localStorage` est touché : `sessionStorage` n'est pas un global de Node et traverse
 * intact, ce qui rendait le symptôme d'autant plus trompeur.
 *
 * Le substitut est posé ici plutôt que contourné test par test : la préférence de son de fin
 * de traitement et celles d'affichage lisent `localStorage` en production, et c'est leur
 * comportement réel qu'il faut pouvoir vérifier. Même registre que `ResizeObserver`
 * ci-dessous — jsdom ne fournit pas ce dont les tests ont besoin, le banc le fournit.
 */
class LocalStorageDeTest implements Storage {
  private readonly entrees = new Map<string, string>();

  get length(): number {
    return this.entrees.size;
  }

  clear(): void {
    this.entrees.clear();
  }

  getItem(cle: string): string | null {
    return this.entrees.get(String(cle)) ?? null;
  }

  key(index: number): string | null {
    return [...this.entrees.keys()][index] ?? null;
  }

  removeItem(cle: string): void {
    this.entrees.delete(String(cle));
  }

  setItem(cle: string, valeur: string): void {
    this.entrees.set(String(cle), String(valeur));
  }
}

Object.defineProperty(globalThis, "localStorage", {
  configurable: true,
  writable: true,
  value: new LocalStorageDeTest(),
});

/**
 * `ResizeObserver` de jsdom.
 *
 * jsdom ne l'implémente pas, et il ne mesure aucun élément : un conteneur responsive de
 * graphique y resterait donc à zéro pixel et ne rendrait rien. Le substitut annonce une
 * taille fixe dès l'observation, ce qui suffit à faire dessiner les séries dans les tests.
 */
const TAILLE_TEST = { width: 640, height: 240 };

class ResizeObserverDeTest implements ResizeObserver {
  constructor(private readonly callback: ResizeObserverCallback) {}

  observe(target: Element): void {
    const contentRect = { ...TAILLE_TEST, top: 0, left: 0, bottom: TAILLE_TEST.height, right: TAILLE_TEST.width, x: 0, y: 0 };
    this.callback(
      [
        {
          target,
          contentRect: contentRect as DOMRectReadOnly,
          borderBoxSize: [{ inlineSize: TAILLE_TEST.width, blockSize: TAILLE_TEST.height }],
          contentBoxSize: [{ inlineSize: TAILLE_TEST.width, blockSize: TAILLE_TEST.height }],
          devicePixelContentBoxSize: [
            { inlineSize: TAILLE_TEST.width, blockSize: TAILLE_TEST.height },
          ],
        },
      ],
      this,
    );
  }

  unobserve(): void {}

  disconnect(): void {}
}

globalThis.ResizeObserver = ResizeObserverDeTest;

// Les mesures directes du DOM sont nulles en jsdom : les composants qui lisent leur propre
// largeur avant le premier `ResizeObserver` obtiennent ainsi la même taille de référence.
Object.defineProperty(HTMLElement.prototype, "offsetWidth", {
  configurable: true,
  get: () => TAILLE_TEST.width,
});
Object.defineProperty(HTMLElement.prototype, "offsetHeight", {
  configurable: true,
  get: () => TAILLE_TEST.height,
});
