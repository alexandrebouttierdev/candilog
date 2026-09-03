import "@testing-library/jest-dom/vitest";

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
