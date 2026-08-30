import { defineConfig, globalIgnores } from "eslint/config";
import nextVitals from "eslint-config-next/core-web-vitals";
import nextTs from "eslint-config-next/typescript";

const eslintConfig = defineConfig([
  ...nextVitals,
  ...nextTs,
  globalIgnores([
    // Ignores par défaut de eslint-config-next.
    ".next/**",
    "out/**",
    "build/**",
    "next-env.d.ts",
    // Handoff de design : dossier de travail, absent du dépôt (voir DESIGN.md).
    // L'ignore reste utile tant qu'une copie locale traîne.
    "design_handoff_candilog_landing/**",
  ]),
]);

export default eslintConfig;
