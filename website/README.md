# Site Candilog

Landing page officielle de Candilog, construite avec Next.js 16 et React 19.

## Démarrage local

```bash
npm install
npm run dev
```

Le site est ensuite disponible sur `http://localhost:3000`.

## Validation

```bash
npm run lint
npm run build
```

Pour régénérer l'image Open Graph après une évolution importante de l'interface :

```bash
npm run generate:og
```

## Captures publiques

Les images de `public/screenshots` sont produites depuis une base isolée qui ne contient que
des personnes, entreprises, coordonnées et URL fictives. Depuis la racine du dépôt :

```bash
DEMO_DIR="$(tools/create_website_demo_database.sh)"
tools/capture_website_demo.sh "$DEMO_DIR"
```

La génération requiert `sqlite3`, ImageMagick et une session graphique locale. Elle n'ouvre
jamais la base utilisateur de Candilog.

## Déploiement

Définir `NEXT_PUBLIC_SITE_URL` avec le domaine public avant le build afin de générer les
métadonnées, `robots.txt` et `sitemap.xml` avec la bonne origine. Le projet produit un serveur
Next.js autonome grâce à `output: "standalone"`.

`NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION` peut recevoir le jeton de validation fourni par Google
Search Console. Après la mise en ligne, soumettre `/sitemap.xml` dans Search Console et contrôler
les données structurées `SoftwareApplication` avec le test des résultats enrichis de Google.

Les boutons de téléchargement utilisent les noms d'assets `-latest` de la release GitHub. Ils
restent donc valides lors des prochaines publications sans modification du site.
