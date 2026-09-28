# candilog.fr

Site officiel de [Candilog](https://github.com/alexandrebouttierdev/candilog) : la landing
page et les quatre pages légales. Export statique, hébergé sur GitHub Pages.

## Démarrer

```bash
npm install
npm run dev        # http://localhost:3000
```

| Commande | Effet |
| --- | --- |
| `npm run dev` | Serveur de développement |
| `npm run build` | Export statique dans `out/` |
| `npm run lint` | ESLint |
| `npm run typecheck` | `tsc --noEmit` |

## Stack

Next.js 16 (App Router) · React 19 · TypeScript strict · Tailwind CSS 4 —
les mêmes versions que l'application desktop.

Aucune dépendance de rendu hors Next et React : les polices IBM Plex (Serif, Mono, Sans)
passent par `next/font`, les icônes sont des tracés SVG inline (`LineIcon`). **Le site ne
fait aucune requête vers un service tiers au chargement** : polices, icônes et logos de
marque sont tous servis depuis le domaine.
C'est vérifiable après un build :

```bash
grep -rhoE 'url\((https?:)?//[^)]+\)' out/_next/static/chunks/*.css   # doit être vide
```

Cette propriété est affirmée dans la politique de confidentialité (§ « Site internet ») :
si vous ajoutez une ressource externe, mettez ce texte à jour.

## Arborescence

```
app/                     layout, globals.css, landing, 4 pages légales,
                         icon.svg, robots.ts, sitemap.ts
components/
  landing/               les sections de la landing (Hero, ProductTour, Workflow,
                         Tracking, DocumentsShowcase, AiSection, Privacy, Faq, DownloadCta)
    app/                 reproductions de l'application : Fenetre (coque), primitives,
      ecrans/            écrans Aujourd'hui, Liste, Documents, Analyse, générateurs
  layout/                SiteHeader, MobileNav, SiteFooter, LegalLayout, ThemeToggle
  legal/                 primitives des pages légales
  ui/                    Button, DownloadMenu, LineIcon, BrandMark, BrandIcon, Reveal
lib/
  data/                  contenus et listes, sortis du JSX ; demo.ts = données fictives
                         des aperçus ; site.ts = titre, description et métadonnées
                         par page
  hooks/                 useScrollReveal
  cn.ts, menuOuvert.ts
public/                  og-image.png, brand/ (marques), providers/ (IA), CNAME
```

Le site applique le design system v2 de l'application desktop (`../docs/DESIGN.md`) :
mêmes jetons, mêmes polices, mêmes composants reproduits dans les aperçus.

## Thème et conventions de style

**[`DESIGN.md`](DESIGN.md) est la référence** : tokens, échelles typographiques, rayons,
durées, comportements responsive, acquis d'accessibilité, contenus juridiquement
calibrés, et le journal des écarts assumés par rapport au prototype d'origine.

Le thème clair/sombre passe par `data-theme` sur `<html>` et des variables CSS, pas par
les variantes `dark:` de Tailwind. `bg-panel` change de valeur tout seul.

Cinq règles, valables pour tout nouveau code :

1. **Aucune couleur Tailwind par défaut.** Pas de `bg-white`, `text-gray-500`,
   `bg-indigo-600`. Les jetons couvrent toute la palette : `bg-panel`, `text-tx-3`,
   `border-bd`, `bg-ac`, `text-on-accent`, `bg-tint-ac-bg`…
2. **Aucune variante `dark:`.** Elle entrerait en conflit avec les tokens.
3. **Les valeurs du design passent par les crochets** — `h-[19px]`, `text-[13.5px]`,
   `px-[7px]`. Ne pas arrondir vers l'échelle Tailwind.
4. **`style={{}}` réservé au dynamique** : une largeur calculée, une rotation d'état, un
   dégradé de marque. Jamais une couleur ou un espacement fixe.
5. **Les variantes vivent dans un objet**, pas dans des ternaires imbriqués au milieu du
   JSX. Voir `GLYPHE` et `TEINTE` dans `components/landing/app/primitives.tsx`.

La feuille A4 des aperçus a ses propres jetons `--paper-*`, identiques dans les deux
thèmes : elle prévisualise une page imprimée.

Le script anti-flash de `app/layout.tsx` pose `data-theme` avant le premier paint. Sans
lui, un visiteur en mode sombre voit un flash blanc.

## Animations

Tout est conditionné à `prefers-reduced-motion` :

| Où | Quoi |
| --- | --- |
| `useScrollReveal` | Apparition au scroll, décalage de 80 ms entre enfants |
| `window-lift` | Fenêtre d'aperçu soulevée de 4 px au survol |
| `animate-pop` | Changement d'écran de la visite produit, fiche du Kanban |

Aucune animation permanente : rien ne bouge sans action ou défilement du visiteur.

## Déploiement

`output: "export"` dans `next.config.ts` — le build produit `out/`, à publier tel quel.
`public/CNAME` porte `candilog.fr`, donc pas de `basePath`. Si le site déménageait sur
`<user>.github.io/<repo>/`, il faudrait renseigner `basePath` et `assetPrefix`.

L'export statique interdit toute route serveur : pas de `/api/download/[platform]`.

## Téléchargements

Les liens de `lib/data/plateformes.ts` pointent sur
`…/releases/latest/download/candilog-<plateforme>-latest.<ext>` (Windows `.exe`,
macOS `.dmg`, Ubuntu `.deb`, Fedora `.rpm`). GitHub sert toujours l'asset du même nom
sur la dernière release publiée.
