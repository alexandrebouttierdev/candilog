import js from "@eslint/js";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";

export default tseslint.config(
  // `src/shared/types/generated` est écrit par ts-rs : le corriger ici serait perdu à la
  // prochaine génération, c'est le Rust qu'il faut modifier.
  {
    ignores: [
      "dist",
      "src-tauri/target",
      "src/shared/types/generated",
      "src-tauri/**",
      // `website/` est un projet Next.js autonome, avec son propre `eslint.config.mjs` et
      // son propre `tsconfig.json` : le linter de l'application n'a pas ses types.
      "website/**",
      "vendor/**",
      "SPECDESIGN/**",
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
  {
    languageOptions: {
      parserOptions: { projectService: true, tsconfigRootDir: import.meta.dirname },
    },
    rules: {
      // Le §36 de MIGRATION.md interdit de reconstruire le backend en TypeScript : les
      // appels IPC passent tous par `shared/services/ipc.ts`, jamais par `invoke` direct.
      "no-restricted-imports": [
        "error",
        {
          paths: [
            {
              name: "@tauri-apps/api/core",
              importNames: ["invoke"],
              message:
                "Passez par `ipc()` de @/shared/services/ipc — les vues et ViewModels n'appellent jamais invoke directement (MIGRATION.md §7).",
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
