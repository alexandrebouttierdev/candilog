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

Deux dépendances de rendu seulement : `material-symbols` (icônes, auto-hébergées) et
JetBrains Mono via `next/font`. **Le site ne fait aucune requête vers un service tiers
au chargement** : polices, icônes et logos de marque sont tous servis depuis le domaine.
C'est vérifiable après un build :

```bash
grep -rhoE 'url\((https?:)?//[^)]+\)' out/_next/static/chunks/*.css   # doit être vide
```

Cette propriété est affirmée dans la politique de confidentialité (§ « Site internet ») :
si vous ajoutez une ressource externe, mettez ce texte à jour.

## Arborescence

```
app/                     layout, globals.css, landing, 4 pages légales
components/
  landing/               les 9 sections de la landing
    JourneyScreens/      les 5 écrans du parcours
  layout/                SiteHeader, SiteFooter, LegalLayout, ThemeToggle
  legal/                 primitives des pages légales
  ui/                    Button, DownloadMenu, Icon, BrandIcon, Reveal
lib/
  data/                  contenus et listes, sortis du JSX
  hooks/                 useScrollReveal, useAtsReveal
  cn.ts, menuOuvert.ts
public/                  logos, brand/ (marques), providers/ (IA), CNAME
```

Le dossier `design_handoff_candilog_landing/` (prototypes `.dc.html` et README
d'intégration) n'a pas vocation à rester dans le dépôt : tout ce qu'il fallait en
retenir est dans [`DESIGN.md`](DESIGN.md).

## Thème et conventions de style

**[`DESIGN.md`](DESIGN.md) est la référence** : tokens, échelles typographiques, rayons,
durées, comportements responsive, acquis d'accessibilité, contenus juridiquement
calibrés, et le journal des écarts assumés par rapport au prototype d'origine.

Le thème clair/sombre passe par `data-theme` sur `<html>` et des variables CSS, pas par
les variantes `dark:` de Tailwind. `bg-surface` change de valeur tout seul.

Cinq règles, valables pour tout nouveau code :

1. **Aucune couleur Tailwind par défaut.** Pas de `bg-white`, `text-gray-500`,
   `bg-indigo-600`. Les tokens couvrent toute la palette : `bg-surface`, `text-ink-muted`,
   `border-line`, `bg-accent`, `text-on-accent`, `bg-tint-10`…
2. **Aucune variante `dark:`.** Elle entrerait en conflit avec les tokens.
3. **Les valeurs du design passent par les crochets** — `h-[19px]`, `text-[13.5px]`,
   `px-[7px]`. Ne pas arrondir vers l'échelle Tailwind.
4. **`style={{}}` réservé au dynamique** : une largeur calculée, une rotation d'état, un
   dégradé de marque. Jamais une couleur ou un espacement fixe.
5. **Les variantes vivent dans un objet**, pas dans des ternaires imbriqués au milieu du
   JSX. Voir `STATUT` dans `lib/data/suivi.ts`.

La bande CV/ATS a son propre jeu de tokens `--band-*`, au contraste volontairement
différent de celui de la page. Ne pas les remplacer par les tokens de page.

Le script anti-flash de `app/layout.tsx` pose `data-theme` avant le premier paint. Sans
lui, un visiteur en mode sombre voit un flash blanc.

## Animations

Trois blocs animés, tous conditionnés à `prefers-reduced-motion` :

| Où | Quoi |
| --- | --- |
| `useScrollReveal` | Apparition au scroll, décalage de 80 ms entre enfants |
| `useAtsReveal` | Score ATS compté à l'entrée dans le viewport, +3 toutes les 26 ms |
| `TrackingBoard` | Boucle de statut toutes les 2 600 ms |

Aucune n'est une animation CSS infinie : elles s'arrêtent, et se nettoient au démontage.

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
