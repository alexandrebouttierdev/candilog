# Design system — candilog.fr

Référence visuelle du site. **Le site n'a pas de design system propre** : il applique
celui de l'application desktop v2 (`../docs/DESIGN.md`, `../reference_design/tokens.json`,
`../src/styles.css`). Une valeur qui n'existe pas dans l'application n'existe pas ici.

Le design est **haute fidélité** : `13,5px` n'est pas `text-sm`, `r9` n'est pas
`rounded-lg`.

---

## 1. Le principe

Les couleurs ne sont pas écrites dans les classes, elles sont **exposées à Tailwind
depuis des variables CSS** (`app/globals.css`, bloc `@theme inline`), sous les noms du
handoff de l'application : `bg-panel`, `text-tx-3`, `border-bd`, `bg-ac`, `bg-st-g`…
`bg-panel` rend `#FFFFFF` en clair et `#16171B` en sombre, sans une seule variante `dark:`.

```tsx
// ✅ une seule écriture, les deux thèmes
<div className="rounded-r9 border border-bd bg-panel text-tx">

// ❌ double maintenance, et ce n'est pas la palette
<div className="rounded-xl border border-gray-200 bg-white dark:bg-gray-900">
```

### Les six règles

1. **Aucune couleur Tailwind par défaut.** Pas de `bg-white`, `text-gray-500`,
   `bg-indigo-600`, ni d'hexadécimal en crochets. Si une couleur manque, elle n'existe pas.
2. **Aucune variante `dark:`.** Elle entre en conflit avec les jetons.
3. **Les valeurs hifi passent par les crochets** : `h-[34px]`, `text-[12.5px]`.
4. **`style={{}}` uniquement pour le dynamique** — une largeur calculée depuis une donnée
   (barre de graphique, segment de répartition). Jamais une couleur ou un espacement fixe.
5. **Les variantes dans un objet**, pas dans des ternaires imbriqués (`GLYPHE`, `TEINTE`,
   `BOUTON`, `AFFICHAGE`…).
6. **Pas de `tailwind-merge`** (`lib/cn.ts`) : un composant ne reçoit jamais une classe
   qui concurrence la sienne. Sa visibilité responsive passe par une prop typée
   (`visible="md"`, `affichage="des-sm"`), sa taille par `taille`/`petit`. `className`
   sert au positionnement.

---

## 2. Jetons

Définis sur `:root` (clair, gris froid — même famille que le sombre) et
`:root[data-theme="dark"]`.

| Famille | Utilitaires |
| --- | --- |
| Surfaces | `bg-app` (fond de page), `bg-panel`, `bg-group`, `bg-elev`, `bg-chip`, `bg-hover`, `bg-sel`, `bg-modal`, `bg-field`, `bg-desk` (bureau sous les fenêtres) |
| Filets | `border-bd`, `border-bd-soft`, `border-bd-menu`, `border-frame`, `border-bd-strong` |
| Encres | `text-tx` → `text-tx-7` (sept niveaux) |
| Accent | `bg-ac`, `bg-ac-h` (survol), `text-ac-tx`, `text-on-accent`, `fill-brand` |
| Statuts | `st-n` · `st-a` · `st-g` · `st-c` — neutre, ambre, vert, rouge |
| Teintes | `bg-tint-{ac,g,c}-bg` + `text-tint-{ac,g,c}-tx` |
| Avatars | `bg-av1` → `bg-av3`, `text-av-tx` |
| Feuille A4 | `bg-paper`, `text-paper-ink{,-2,-3}`, `bg-paper-rule`, `bg-paper-sk`, `bg-paper-side`, `bg-paper-mark` — encres des gabarits de l'application, **identiques dans les deux thèmes** |
| Fenêtre | `bg-wc1` → `bg-wc3` (feux macOS, gris en sombre) |

**Jetons v1 hérités.** `bg-page`, `bg-surface`, `text-ink*`, `border-line`,
`border-control`, `text-accent*`… sont **repointés** sur la palette v2 : les pages
légales et la 404 les emploient encore. Tout code neuf utilise les noms v2.

| Rayons | Ombres | Courbes |
| --- | --- | --- |
| `rounded-r2` → `rounded-r12` (px) | `shadow-window` (fenêtres d'aperçu) | `ease-out-soft` — `cubic-bezier(.2,.7,.2,1)` |
| | `shadow-pop` (menus, dialogues, fiche flottante) | `ease-reveal` — `cubic-bezier(.16,1,.3,1)` |
| | `shadow-sheet` (feuille A4) | |

**Aucune ombre sur le contenu.** La hiérarchie vient des surfaces et des filets de 1 px.

---

## 3. Thème clair / sombre

- Piloté par `data-theme="light" | "dark"` sur `<html>`, mémorisé sous `candilog-theme`,
  `prefers-color-scheme` en repli au premier chargement.
- **Le script anti-flash de `app/layout.tsx` est obligatoire** : il pose l'attribut
  avant le premier paint.
- `ThemeToggle` : bouton carré de 32 px, icônes au trait `sun` / `moon`.
- La feuille A4 reste claire en sombre : elle prévisualise une page imprimée.

---

## 4. Typographie et icônes

| Usage | Famille |
| --- | --- |
| Texte courant, interface | `system-ui, -apple-system, "Segoe UI"` |
| Titres (H1, H2, titres d'écran, scores) | **IBM Plex Serif** 600, `serif-title` (interlettrage `-0.02em`) |
| Références, dates, compteurs, touches | **IBM Plex Mono** (`font-mono`) |
| Feuille A4 (CV, lettre) | **IBM Plex Sans** (`font-plex`), comme le PDF exporté |

Les trois familles passent par `next/font/google` : téléchargées au build, servies
depuis le site.

| Titre | Valeur |
| --- | --- |
| H1 (hero) | 34 → 44 → 54 px, `leading-[1.04]`, `-0.03em` |
| H2 de section | 26 → 32 → 36 px, `-0.025em` |
| H2 de clôture (`DownloadCta`) | 28 → 40 px, `-0.028em` |
| H2 de la FAQ | 26 → 32 px |
| H3 éditorial | 21 → 24 px |

L'échelle a été **descendue d'environ un cinquième** : à 66 px, le H1 occupait deux lignes
de 145 px et repoussait l'application sous la ligne de flottaison, et les H2 à 44 px pesaient
plus que les aperçus qu'ils annonçaient. Les tailles des **aperçus** (`landing/app/`) n'en
dépendent pas : elles copient l'application et ne bougent qu'avec elle (§10).
| Sur-titre (`eyebrow`) | 12 px, capitales, `0.08em`, `tx-4` |
| En-tête d'aperçu (`caps`) | 10,5 px, capitales, `0.04em`, `tx-6` — celui de l'application |

**Icônes : aucune police d'icônes** (décision D7 de l'application). `LineIcon` porte les
onze tracés de l'application (grille 16, trait 1,4 px, `currentColor`) et quelques
tracés propres au site dessinés sur la même grille. Les logos de marque passent par
`BrandIcon` (SVG en `mask-image`, couleur du texte). La marque est `BrandMark`, la
tuile `#5B62F0` de l'application ; `app/icon.svg` en est la copie.

---

## 5. Layout et responsive

- Contenu : **1200 px** centré ; gouttières 16 px (mobile), 32 px (dès 768 px).
- **Hero sur une seule colonne** : titre, chapô (620 px au plus) et boutons se suivent
  verticalement. La version en deux colonnes écartait l'appel à l'action de 80 px, collé au
  bord droit, loin de la phrase qui le justifie. C'est la fenêtre d'application en dessous,
  pleine largeur, qui occupe les 1200 px.
- Sections : 64 → 96 → 128 px de marge verticale. En-tête de section éditorial
  (`EnTeteSection`) : titre à gauche, chapô à droite dès 1024 px.
- Contrôles : 32 px compact, 40 px principal, **48 px tactile** sous 640 px.

Les aperçus ne sont **pas réduits** : ils se recomposent.

| Largeur | Fenêtre d'aperçu |
| --- | --- |
| < 768 px | Pas de navigation latérale ni de feux : un recadrage de l'écran, hauteur libre. Colonnes secondaires masquées, CV et lettre montrés en haut de feuille puis le panneau de score dessous. |
| 768–1023 px | Navigation repliée en icônes (52 px), comme l'application sous 1060 px. |
| ≥ 1024 px | Navigation complète (202 px), hauteur fixe. Fiche latérale dès 1280 px. |

Défilements horizontaux **structurels** : Kanban sous 1024 px (colonnes aimantées),
frise du parcours sous 1280 px. Aucun débordement horizontal de page (vérifié à 375,
768, 1024 et 1440 px).

---

## 6. Animations

Sobres, bornées, jamais infinies. Toutes neutralisées par `prefers-reduced-motion`.

| Où | Comportement |
| --- | --- |
| `Reveal` / `useScrollReveal` | Montée de 16 px + fondu, 80 ms entre enfants, seuil 0,08. Le rendu serveur sort visible. |
| `window-lift` | Fenêtre d'aperçu soulevée de 4 px au survol (pointeur fin seulement). |
| `animate-pop` | Changement d'écran de la visite produit, ouverture de la fiche du Kanban (240 ms). |
| FAQ | `grid-template-rows: 0fr → 1fr`, 320 ms. |

> ⚠️ `.reveal-in` **libère** le `will-change` posé par `.reveal` : sous Chromium il fait
> de l'élément une *backdrop root*. Ne pas le remettre.

---

## 7. Accessibilité — acquis à conserver

- Lien d'évitement « Aller au contenu », un seul `h1`, un `h2` par section.
- `:focus-visible { outline: 1px solid var(--ac) }` global.
- Cibles ≥ 44 px sur mobile (menu, liens de pied de page, onglets).
- Aperçus décoratifs : `aria-hidden`, zéro élément focusable ; leur contenu est décrit
  par un `figcaption` masqué. **Exception : le Kanban**, dont les cartes sont de vrais
  boutons (`aria-pressed`) ouvrant une fiche (`aside` nommée) ; `Échap` ou « ✕ »
  referment et rendent le focus à la carte.
- Visite produit : vrai `tablist`, tabindex itinérant, flèches et Début/Fin.
- Menus (téléchargement, navigation mobile) et FAQ : *disclosures* avec `aria-expanded`
  + `aria-controls`, panneau fermé `inert`, fermeture à `Échap` et au clic extérieur.
  Dans la FAQ, la question reste le texte du `<h3>` et le bouton, étiré sur la ligne,
  est nommé par `aria-labelledby` : un titre vide pour les extracteurs de contenu
  (qui ignorent l'intérieur des boutons) nuisait à l'indexation.
- Statuts : glyphe de forme (vide, demi, trois quarts, plein) **et** libellé, jamais la
  couleur seule.

---

## 8. SEO

- Métadonnées par défaut dans `app/layout.tsx` (titre, description, mots-clés, Open
  Graph, Twitter, `theme-color`). Chaque page déclare les siennes avec
  `metadonneesPage()` (`lib/data/site.ts`) : titre, description, canonique, Open Graph
  et Twitter propres, pour qu'aucune page ne reprenne ceux de l'accueil.
- Données structurées (JSON-LD) dans `app/page.tsx` : `SoftwareApplication` et
  `FAQPage`, construite depuis `lib/data/faq.tsx` (questions et réponses à l'identique).
- `public/og-image.png` (1200 × 630) : image de partage statique — un fichier à
  extension, servi avec le bon type MIME par GitHub Pages.
- `app/robots.ts`, `app/sitemap.ts` : générés au build.

---

## 9. Contenus juridiquement calibrés

> ⚠️ Les textes des 4 pages légales, et les réponses de FAQ portant sur la
> confidentialité, la licence et l'ATS, sont **calibrés**. Ils ne promettent ni
> « aucune donnée ne quitte votre ordinateur », ni un résultat de recrutement. Ne pas
> les reformuler sans relecture. La section Confidentialité de la landing reprend la
> réponse calibrée de la FAQ.

Mentions obligatoires à conserver telles quelles :

- « Une analyse est une indication, pas une garantie de sélection. » (section Documents)
- Le § « Site internet » de la politique de confidentialité affirme que le site ne fait
  aucune requête vers un service tiers. **Ajouter une ressource externe rend ce texte
  faux** — mettre le texte à jour, ou renoncer à la ressource.

---

## 10. Fidélité des aperçus

Les aperçus — Aujourd'hui, Candidatures, Documents, Analyse, Kanban, générateurs de CV
et de lettre, analyse face à l'offre, écran IA, dialogue d'envoi distant — ne sont pas
des illustrations libres. **Ils ne montrent que ce que l'application fait.**

### La règle

`../src/` et `../reference_design/screens/` font foi. Avant de modifier un aperçu, lire
le composant réel et en reprendre la structure, les libellés et les états.

| Aperçu (`components/landing/app/`) | Source de vérité |
| --- | --- |
| `Fenetre` (coque, navigation, barres) | `src/app/layout/`, `docs/DESIGN.md` §8 |
| `primitives` (glyphe, pastille, avatar, touche) | `src/shared/ui/{StatusGlyph,Tag,Avatar,Kbd}.tsx` |
| `EcranAujourdhui` | `screens/01-today-light.png` |
| `EcranListe`, Kanban (`Tracking`) | `screens/02`, `03`, `19` ; `features/applications/model/statuses.ts` |
| `EcranDocuments` | `screens/07-resumes.png` |
| `EcranAnalyse` | `screens/10-analytics.png` |
| `Generateurs` | `screens/15`, `09`, `16` |
| Écran IA (`AiSection`) | `screens/13` ; `features/settings/model/{providers,taskRouting}.ts` |
| Dialogue d'envoi (`Privacy`) | `features/settings/view/components/RemoteSendDialog.tsx` |

### Ce que l'application ne fait pas

À ne pas mettre en scène, quelle que soit la qualité graphique du résultat :

- **Pas de cinquième statut.** Quatre : En attente, Relancée, Entretien, Refusée. Ni
  « À postuler » ni « Offre reçue ».
- **Pas d'import d'annonce par URL** : une offre entre par son texte collé.
- **Pas de pièces jointes** sur une candidature : les documents vivent dans Documents.
- **Pas de pourcentage de progression** sur un traitement IA, pas de flèche de tendance
  dans l'analyse (aucune période de comparaison n'existe).
- L'IA locale s'appelle **« IA locale »**, jamais d'après une famille de modèles.

### Le jeu de données

Tout vient de `lib/data/demo.ts`. Un seul persona — **Camille Berthier**, designer
produit à Lyon — et des entreprises fictives (Atelier Nord, Studio Halage, Cobalt
Bureau, Groupe Vallée, Sablé Industries, Éditions Sillon, Nord Réseaux, Maison Rivet,
Laurier & Pons, Verrières & Cie). Date du jour des aperçus : lundi 28 septembre 2026.
Compteurs cohérents entre sections : **14 candidatures**, 5 / 3 / 3 / 3. Contact sur
`exemple.fr`, téléphone de fiction.
