import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import jsxA11y from "eslint-plugin-jsx-a11y";

export default tseslint.config(
  // `src/shared/types/generated` est écrit par ts-rs : le corriger ici serait perdu à la
  // prochaine génération, c'est le Rust qu'il faut modifier.
  {
    ignores: [
      "dist",
      "src-tauri/target",
      "test-output",
      "src/shared/types/generated",
      "src-tauri/**",
      // `website/` est un projet Next.js autonome, avec son propre `eslint.config.mjs` et
      // son propre `tsconfig.json` : le linter de l'application n'a pas ses types.
      "website/**",
      "vendor/**",
      "docs/**",
      ".npm-cache/**",
      ".pnp.cjs",
      ".pnp.loader.mjs",
      ".yarn/**",
    ],
  },
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  reactHooks.configs.flat["recommended-latest"],
  // L'accessibilité est soignée à la main — rôles, `aria-live`, `aria-activedescendant`,
  // piège de focus, `GlyphButton` qui exige un `label` — mais rien ne la protégeait d'une
  // régression. Ce plugin verrouille donc la trentaine de règles que le code respecte déjà :
  // `alt-text`, `aria-props`, `aria-role`, `role-has-required-aria-props`,
  // `aria-activedescendant-has-tabindex`, `tabindex-no-positive`, `img-redundant-alt`…
  jsxA11y.flatConfigs.recommended,
  {
    rules: {
      // Candilog gère le clavier **au conteneur**, pas sur chaque élément : une liste porte
      // `role="listbox"` et `aria-activedescendant`, le `onKeyDown` vit sur le champ ou sur
      // la liste, et les options ne sont ni tabulables ni porteuses de leur propre
      // gestionnaire — c'est le motif ARIA correct (palette `⌘K`, Menu, Aujourd'hui,
      // Documents, Relations, groupes de candidatures, onglets du Profil, « Qui fait quoi »).
      // Ces quatre règles inspectent l'élément seul et ne peuvent pas voir le conteneur :
      // les satisfaire demanderait d'ajouter des gestionnaires redondants et de rendre
      // tabulable ce que le motif veut justement sortir de l'ordre de tabulation.
      // `aria-activedescendant-has-tabindex`, qui garde ce motif, reste active.
      "jsx-a11y/click-events-have-key-events": "off",
      "jsx-a11y/interactive-supports-focus": "off",
      "jsx-a11y/no-noninteractive-element-interactions": "off",
      "jsx-a11y/no-noninteractive-tabindex": "off",
      // Garde de focus et non interaction : la barre d'outils de l'éditeur de lettre
      // intercepte `mousedown` pour ne pas perdre la sélection du texte, les contrôles
      // interactifs étant les boutons qu'elle contient.
      "jsx-a11y/no-static-element-interactions": "off",
      // Règle écrite contre l'autofocus au chargement d'une page. Ici il ne concerne que des
      // surfaces que l'utilisateur vient d'ouvrir — palette, dialogues, menu de filtres —
      // où placer le focus sur le premier champ est la pratique recommandée, et où
      // `useFocusTrap` le retient déjà.
      "jsx-a11y/no-autofocus": "off",
      // Le libellé enveloppe son `input` et son texte vit dans un `<span>` imbriqué, donc
      // au-delà de la profondeur de recherche par défaut (2).
      "jsx-a11y/label-has-associated-control": ["error", { depth: 4 }],
    },
  },
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      // `docs/CODE_RULES.md` §4 : les appels IPC passent tous par
      // `shared/services/ipc.ts`, jamais par `invoke` direct.
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              importNames: ["invoke"],
              message:
                "Passez par `ipc()` de @/shared/services/ipc — les vues et ViewModels n'appellent jamais invoke directement (docs/CODE_RULES.md §4).",
            },
          ],
        },
      ],
    },
  },
  {
    files: ["src/shared/services/ipc.ts"],
    rules: { "no-restricted-imports": "off" },
  },
  // Fichiers de configuration hors du programme TypeScript : les règles à typage requis
  // n'ont pas de types à consulter et échoueraient au parsing.
  {
    files: ["eslint.config.js"],
    extends: [tseslint.configs.disableTypeChecked],
  },
);
