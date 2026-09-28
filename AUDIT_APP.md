# Procédure d'audit technique de Candilog

Prompt à donner tel quel à un agent (Claude Code, Codex, Cursor…) pour rejouer l'audit
technique complet de l'application.

## Sortie attendue

L'audit produit **un seul fichier** : `RAPPORT_AUDIT.md`, à la racine du dépôt.

- Le créer s'il n'existe pas ; le **remplacer entièrement** s'il existe déjà (le rapport
  décrit l'état du code au moment de l'audit, pas un historique).
- Il est ignoré par Git (`.gitignore`) : ne jamais le commiter ni le forcer avec `git add -f`.
- C'est la **seule exception** à la règle « Ne modifie aucun fichier » ci-dessous : aucun
  autre fichier du dépôt ne doit être créé, modifié ou supprimé. Les sorties des commandes
  (logs de tests, d'audit de dépendances) vont dans un dossier temporaire hors du dépôt.
- En tête du rapport, indiquer : date de l'audit, branche, commit (`git rev-parse --short HEAD`)
  et état de `git status` avant et après l'audit.
- À la fin de l'audit, vérifier que `git status --short` ne montre aucune modification
  (hors fichiers déjà ignorés), et le mentionner dans le rapport.
- Dans la réponse de fin, résumer les constats principaux et donner le chemin du rapport.

---

Je veux que tu réalises un **audit technique complet de l'application Candilog**.

## 1. Documentation à lire en priorité

Commence par **lire intégralement tous les fichiers présents dans le dossier `docs/`**.

Ne te limite pas aux noms des fichiers ou aux résumés : comprends précisément :

* l'architecture prévue ;
* les conventions de développement ;
* le design system ;
* les règles de code ;
* les choix techniques ;
* les règles concernant Rust, React, TypeScript et Tauri ;
* les exigences de sécurité ;
* les décisions d'architecture ;
* les fonctionnalités attendues.

Ensuite, analyse le code source de l'application afin de vérifier si l'implémentation respecte réellement cette documentation.

## 2. Important : audit uniquement

Pour cette étape :

**NE MODIFIE AUCUN FICHIER** (seule exception : l'écriture de `RAPPORT_AUDIT.md`, voir « Sortie attendue »).

Je veux uniquement un audit approfondi et un rapport.

Ne corrige pas automatiquement les problèmes trouvés.

Si tu identifies un problème, explique :

* où il se trouve ;
* pourquoi c'est un problème ;
* son impact ;
* sa sévérité ;
* comment il pourrait être corrigé.

---

# 3. Audit global de l'application

Analyse l'ensemble du projet, notamment :

* `src/`
* `src-tauri/`
* `docs/`
* configuration Tauri
* configuration TypeScript
* configuration React
* dépendances
* scripts
* tests
* fichiers de configuration
* architecture générale

Adapte l'analyse à la structure réelle du projet et ne suppose pas qu'un fichier existe s'il n'existe pas.

---

# 4. Architecture

Vérifie précisément :

* respect de l'architecture définie dans `docs/` ;
* séparation des responsabilités ;
* découplage des couches ;
* dépendances entre modules ;
* responsabilités des composants ;
* responsabilités des services ;
* gestion de l'état ;
* gestion des effets de bord ;
* communication React ↔ Tauri ;
* communication TypeScript ↔ Rust ;
* organisation des features ;
* risques de dépendances circulaires ;
* code dupliqué ;
* logique métier placée au mauvais endroit ;
* cohérence des abstractions.

Identifie également les endroits où l'architecture risque de devenir difficile à maintenir avec la croissance de l'application.

---

# 5. Audit React / TypeScript

Effectue un audit approfondi du frontend.

Analyse notamment :

### React

* qualité des composants ;
* taille et complexité des composants ;
* séparation présentation / logique ;
* hooks ;
* custom hooks ;
* gestion de l'état ;
* effets `useEffect` ;
* dépendances des hooks ;
* re-renders inutiles ;
* gestion des formulaires ;
* gestion des erreurs ;
* accessibilité ;
* composants réutilisables ;
* duplication ;
* cohérence avec le design system.

### TypeScript

* qualité du typage ;
* `any` ;
* casts abusifs ;
* types trop permissifs ;
* types dupliqués ;
* interfaces incohérentes ;
* nullability ;
* gestion des erreurs ;
* contrats entre frontend et backend Rust ;
* cohérence des DTO ;
* utilisation correcte de Zod si présent.

Signale les problèmes de typage qui pourraient provoquer des bugs à runtime.

---

# 6. Audit Rust

Effectue un audit spécifique et approfondi du code Rust présent dans `src-tauri/`.

Analyse notamment :

* idiomaticité Rust ;
* ownership / borrowing ;
* gestion des `Result` et `Option` ;
* propagation des erreurs ;
* `unwrap()` / `expect()` ;
* `panic!` ;
* gestion des erreurs ;
* sécurité mémoire ;
* concurrence ;
* synchronisation ;
* gestion des ressources ;
* performances ;
* clonages inutiles ;
* allocations inutiles ;
* complexité ;
* organisation des modules ;
* séparation des responsabilités ;
* API publiques ;
* sérialisation / désérialisation ;
* commandes Tauri ;
* validation des entrées ;
* accès filesystem ;
* accès réseau ;
* accès base de données.

Vérifie également les risques spécifiques liés à une application desktop locale.

---

# 7. Audit Tauri

Analyse spécifiquement l'utilisation de Tauri.

Vérifie :

* architecture Tauri ;
* commandes `invoke` ;
* paramètres des commandes ;
* validation des entrées provenant du frontend ;
* permissions ;
* capabilities ;
* allowlist / configuration selon la version de Tauri utilisée ;
* exposition des commandes Rust ;
* surface d'attaque ;
* accès filesystem ;
* accès processus ;
* accès réseau ;
* gestion des fenêtres ;
* événements Tauri ;
* communication frontend/backend ;
* gestion des erreurs ;
* configuration de production ;
* sécurité du bundle ;
* configuration des CSP si applicable ;
* pratiques recommandées par la version actuelle de Tauri utilisée.

Identifie toute permission excessive ou fonctionnalité exposée inutilement.

---

# 8. Sécurité

Effectue un véritable audit de sécurité.

Recherche notamment :

* injection ;
* XSS ;
* command injection ;
* path traversal ;
* accès filesystem dangereux ;
* exécution de commandes système ;
* secrets exposés ;
* clés API dans le code ;
* variables d'environnement mal utilisées ;
* données sensibles dans les logs ;
* validation insuffisante des entrées ;
* permissions Tauri trop larges ;
* communications réseau non sécurisées ;
* stockage local de données sensibles ;
* mauvaise gestion des fichiers importés ;
* risques liés aux documents CV ;
* risques liés aux appels aux modèles IA ;
* risques liés aux prompts utilisateur ;
* SSRF si applicable ;
* dépendances vulnérables.

Pour chaque problème de sécurité, indique :

**Sévérité : Critical / High / Medium / Low / Informational**

Et explique précisément le scénario d'exploitation potentiel.

---

# 9. Dépendances et supply chain

Analyse :

* `package.json`
* lockfiles
* `Cargo.toml`
* `Cargo.lock`
* dépendances Tauri
* dépendances React
* dépendances Rust

Recherche :

* dépendances obsolètes ;
* dépendances inutilisées ;
* dépendances dupliquées ;
* dépendances présentant des vulnérabilités connues ;
* packages à risque ;
* dépendances inutilement lourdes.

Utilise les outils adaptés du projet (`npm`, `pnpm`, `yarn`, `cargo`, `cargo audit`, etc.) lorsque disponibles.

---

# 10. Utilisation de Context7

**Utilise Context7 systématiquement lorsque tu dois vérifier une API, une pratique ou une recommandation concernant une librairie ou framework.**

En particulier pour :

* React ;
* TypeScript ;
* Tauri ;
* Rust crates importantes ;
* bibliothèques utilisées par le projet.

Ne te base pas uniquement sur tes connaissances internes lorsqu'une documentation officielle à jour peut être consultée via Context7.

Compare ensuite l'implémentation actuelle avec les recommandations correspondant aux **versions réellement utilisées dans le projet**.

---

# 11. Tests

Audit complet des tests :

* couverture fonctionnelle ;
* tests unitaires ;
* tests d'intégration ;
* tests frontend ;
* tests Rust ;
* tests des commandes Tauri ;
* tests des validations ;
* tests des cas d'erreur ;
* tests des cas limites ;
* tests des fonctionnalités critiques.

Identifie :

* fonctionnalités sans tests ;
* tests trop superficiels ;
* tests fragiles ;
* duplication ;
* mocks excessifs ;
* tests qui ne vérifient pas réellement le comportement attendu.

Si possible, exécute les suites de tests existantes afin de vérifier leur état réel.

**Ne modifie aucun test pendant l'audit.**

---

# 12. Qualité et maintenabilité

Analyse :

* lisibilité ;
* complexité cyclomatique ;
* fonctions trop longues ;
* fichiers trop volumineux ;
* duplication ;
* naming ;
* cohérence des conventions ;
* commentaires inutiles ou insuffisants ;
* dette technique ;
* abstractions prématurées ;
* couplage ;
* cohésion ;
* facilité d'ajout de nouvelles fonctionnalités ;
* facilité de debugging.

Identifie les zones qui représentent actuellement les principaux risques de maintenance.

---

# 13. Performance

Analyse les problèmes potentiels de performance :

### React

* re-renders ;
* calculs inutiles ;
* listes ;
* memoization ;
* chargement initial ;
* bundle ;
* images/assets ;
* gestion de l'état.

### Rust / Tauri

* opérations bloquantes ;
* I/O ;
* filesystem ;
* base de données ;
* appels synchrones ;
* allocations ;
* traitement de fichiers ;
* communication IPC.

Ne propose pas d'optimisations prématurées : distingue les problèmes mesurables des optimisations simplement potentielles.

---

# 14. Base de données / persistance

Si l'application utilise SQLite ou une autre base locale, vérifie :

* architecture d'accès aux données ;
* migrations ;
* transactions ;
* requêtes ;
* paramètres SQL ;
* injections SQL ;
* gestion des erreurs ;
* concurrence ;
* cohérence des données ;
* intégrité ;
* index ;
* performances ;
* séparation repository / logique métier.

---

# 15. Configuration et production

Vérifie que le projet est correctement préparé pour une application desktop distribuée aux utilisateurs.

Analyse :

* configuration dev / production ;
* build ;
* bundle ;
* variables d'environnement ;
* secrets ;
* logs ;
* crash handling ;
* mises à jour si présentes ;
* permissions ;
* configuration Tauri ;
* packaging ;
* sécurité du build.

---

# 16. Analyse de cohérence avec la documentation

Après l'audit du code, compare systématiquement :

**Documentation → Architecture → Implémentation → Tests**

Identifie :

* ce qui est conforme ;
* ce qui est partiellement conforme ;
* ce qui n'est pas conforme ;
* ce qui est documenté mais non implémenté ;
* ce qui est implémenté mais non documenté ;
* les règles présentes dans `docs/` mais non respectées.

---

# 17. Exécution des outils

Lorsque c'est possible, utilise les outils disponibles pour vérifier objectivement le projet :

* tests ;
* lint ;
* typecheck ;
* build ;
* audit des dépendances ;
* outils Rust ;
* outils TypeScript ;
* outils React ;
* outils Tauri.

Ne te contente pas d'une analyse statique si une vérification automatisée est possible.

---

# 18. Rapport final

À la fin, produis un rapport d'audit structuré, **écrit dans `RAPPORT_AUDIT.md` à la racine du dépôt** (voir « Sortie attendue »).

## Résumé exécutif

Donne une synthèse claire de l'état actuel du projet.

Ne donne pas une note arbitraire du type `8/10`.

À la place, présente les principaux constats et les principaux risques.

## Tableau des problèmes

Utilise ce format :

| ID | Catégorie | Sévérité | Fichier | Problème | Impact |
| -- | --------- | -------- | ------- | -------- | ------ |

Classe les problèmes par sévérité :

1. Critical
2. High
3. Medium
4. Low
5. Informational

## Détails

Pour chaque problème :

### [ID] Titre

**Sévérité :**
**Catégorie :**
**Fichier(s) :**
**Localisation :**

**Problème :**
Explication précise.

**Impact :**
Conséquences possibles.

**Recommandation :**
Solution recommandée sans modifier le code.

---

# 19. Plan de correction

Termine par un plan de correction priorisé :

### Phase 1 — Critical

Problèmes qui doivent être traités avant toute autre chose.

### Phase 2 — High

Problèmes importants à traiter rapidement.

### Phase 3 — Medium

Améliorations importantes de qualité et maintenabilité.

### Phase 4 — Low

Dette technique et améliorations secondaires.

### Phase 5 — Improvements

Améliorations optionnelles.

Pour chaque élément, indique :

* problème ;
* fichiers concernés ;
* difficulté estimée ;
* dépendances éventuelles ;
* ordre recommandé.

---

# 20. Règles importantes

* **Ne modifie aucun fichier** (seule exception : `RAPPORT_AUDIT.md`).
* **Ne supprime aucun fichier.**
* **Ne refactore rien.**
* **Ne corrige rien automatiquement.**
* Lis d'abord toute la documentation `docs/`.
* Vérifie les affirmations techniques avec Context7 lorsque pertinent.
* Base les conclusions sur le code réellement présent.
* Ne considère pas qu'une pratique est mauvaise simplement parce qu'elle est différente de ta préférence personnelle.
* Prends en compte les versions réellement utilisées.
* Distingue les problèmes avérés des améliorations potentielles.
* Ne cache aucun problème découvert.
* Signale également ce qui est correctement implémenté.
* Si une information est impossible à vérifier, indique-le explicitement.

L'objectif est d'obtenir un **audit technique professionnel et exploitable de Candilog**, couvrant architecture, React/TypeScript, Rust, Tauri, sécurité, tests, performances, dépendances et maintenabilité.
