<!-- BEGIN:nextjs-agent-rules -->

# This is NOT the Next.js you know

This version has breaking changes — APIs, conventions, and file structure may all differ from your training data. Read the relevant guide in `node_modules/next/dist/docs/` (resolved from this file's directory; in monorepos the `next` package may not be visible from the repo root) before writing any code. Heed deprecation notices.

This block is written and re-added by `next dev` — verify at `node_modules/next/dist/server/lib/generate-agent-files.js`. Removing it from a diff only re-creates the uncommitted change; committing it with your work keeps the tree clean.

<!-- END:nextjs-agent-rules -->

---

# Site candilog.fr — règles spécifiques

Projet **autonome** : dépendances, ESLint, TypeScript et commandes propres. Les règles de
l'application desktop (`../AGENTS.md`) ne s'appliquent pas ici, à l'exception des règles
générales de comportement et de la sécurité Git.

## Documentation

- `README.md` — stack, arborescence, déploiement, liens de téléchargement.
- `DESIGN.md` — **référence visuelle** : tokens, échelles, rayons, animations,
  accessibilité. À lire avant de toucher au style.

## Règles de style (résumé, détail dans `DESIGN.md`)

1. Aucune couleur Tailwind par défaut (`bg-white`, `text-gray-500`…) : uniquement les
   tokens du thème.
2. Aucune variante `dark:` — le thème passe par `data-theme` et des variables CSS.
3. Les valeurs haute fidélité passent par les crochets (`h-[19px]`, `text-[13.5px]`).
4. `style={{}}` réservé au dynamique, jamais à une couleur ou un espacement fixe.
5. Les variantes vivent dans un objet, pas dans des ternaires imbriqués.

La bande CV/ATS a son propre jeu de tokens `--band-*` : ne pas la repeindre avec les
tokens de page.

## Contraintes

- Export statique (`output: "export"`) : aucune route serveur, aucun handler d'API.
- **Aucune requête vers un service tiers au chargement** : polices, icônes et logos sont
  auto-hébergés. Cette propriété est affirmée dans la politique de confidentialité ;
  ajouter une ressource externe oblige à mettre ce texte à jour.
- Les liens de téléchargement (`lib/data/plateformes.ts`) pointent sur les assets
  `-latest` des GitHub Releases — voir `../docs/RELEASES.md`.

## Validation

Depuis `website/` :

```bash
npm run lint
npm run typecheck
npm run build
```
