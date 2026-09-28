# Contexte produit pour Claude Design — refonte UX/UI de Candilog

Document de **contexte fonctionnel** destiné à une refonte visuelle complète de Candilog.
Il décrit ce que l'application **fait**, pas comment elle est habillée aujourd'hui.

## Statut du design actuel : LEGACY

L'interface existante est considérée comme **obsolète et non contraignante**. Ne pas
reprendre, ne pas s'inspirer de, ne pas préserver :

- le rail de navigation vertical 68 px et sa logique de 7 tuiles ;
- la topbar 46 px et ses accessoires à droite ;
- la sous-navigation 186 px en colonne ;
- les cartes, filets 1 px, surfaces « glass », densités et rayons actuels ;
- la palette (accent indigo `#4f5fe8` / `#6b7cff`, gris `#f2f3f6` / `#08090c`) ;
- l'échelle typographique actuelle (corps 12,5 px, contrôles 30 px) ;
- les classes Tailwind sémantiques existantes (`bg-surface`, `text-ink`, `border-line`…) ;
- les layouts maître-détail / inspecteur / trois colonnes tels qu'ils sont découpés ;
- `docs/DESIGN.md`, qui décrit le système legacy et sera réécrit après la refonte.

**On conserve les fonctionnalités, les données et les parcours. Pas l'interface.**

Une seule contrainte visuelle survit, pour une raison fonctionnelle : les **aperçus de
documents** (CV et lettre) doivent rester des feuilles A4 blanches fidèles au PDF exporté
(voir §6).

---

## 1. Ce qu'est Candilog

**Application desktop Tauri 2** (macOS / Linux ; Windows partiellement) de suivi de
recherche d'emploi, avec assistance IA locale ou distante.

- **100 % local.** SQLite sur la machine de l'utilisateur, aucun compte, aucun serveur,
  aucune synchronisation. Les seules sorties réseau sont : l'appel au fournisseur IA choisi
  par l'utilisateur, la vérification de mise à jour, et l'ouverture de liens externes.
- **Mono-utilisateur, mono-fenêtre.** Pas de multi-compte, de rôles, de partage, de
  collaboration, de notifications push.
- **Outil de travail quotidien**, utilisé au clavier, plusieurs fois par jour, sur une
  fenêtre native large. Pas un produit à vendre : aucun écran n'a à convaincre.
- **Interface en français.** Les identifiants techniques restent en anglais.
- Thème **clair et sombre** obligatoires, plus un mode « système ».

### Les 8 domaines fonctionnels

| Domaine | Contenu |
| --- | --- |
| **Candidatures** | L'objet central : un poste visé chez une entreprise, avec un statut de pipeline |
| **Entreprises** | Répertoire des sociétés ciblées, qualifiées (secteur, type, taille) |
| **Réseau** | Contacts professionnels, rattachés ou non à une entreprise |
| **Entretiens & relances** | Événements datés rattachés à une candidature, vus dans un calendrier |
| **Documents** | CV ciblés et lettres de motivation générés, édités, exportés en PDF |
| **Profil** | Bibliothèque de faits professionnels : source de vérité de tous les documents |
| **Statistiques** | Métriques, funnel, activité hebdomadaire, candidatures à relancer |
| **IA** | Fournisseurs, modèles, benchmark, progression et annulation des traitements |

---

## 2. Écrans existants

17 écrans réels, regroupés en 7 sections. Le regroupement actuel n'est **pas** une
contrainte : la refonte peut réorganiser la navigation, tant que toutes les fonctions
restent atteignables.

| # | Écran | Route actuelle | Rôle |
| --- | --- | --- | --- |
| 1 | **Aujourd'hui** | `/` | Tableau de bord du jour : prochain événement, à faire, récentes, activité, pipeline |
| 2 | **Candidatures** | `/tracking/applications` | Kanban ou table, filtres, fiche latérale, création/édition |
| 3 | **Calendrier** | `/tracking/calendar` | Entretiens et relances, vues Mois / Semaine / Jour |
| 4 | **Entreprises** | `/relations/companies` | Liste paginée + fiche avec candidatures liées et métriques |
| 5 | **Réseau** | `/relations/network` | Liste paginée de contacts + fiche |
| 6 | **Mes CV** | `/documents/cv` | Bibliothèque de versions + aperçu A4 + actions |
| 7 | **Générer un CV** | `/documents/generate-resume` | Brief d'offre → génération IA → éditeur A4 + panneau d'aide au contenu |
| 8 | **Mes lettres** | `/documents/cover-letters` | Bibliothèque de lettres + aperçu A4 + actions |
| 9 | **Lettre de motivation** | `/documents/write-cover-letter` | Brief → rédaction IA → éditeur A4 + itérations |
| 10 | **Analyse de CV** | `/documents/analyze` | PDF externe + offre → score ATS + recommandations |
| 11 | **Statistiques** | `/analytics` | KPI, graphiques, funnel, à relancer, performance |
| 12 | **Profil** | `/profile` | 10 sections éditables + complétude + import IA depuis un CV |
| 13 | **Réglages → IA** | `/settings/ai` | IA locale (catalogue de modèles) / IA distante (fournisseurs, clés, modèles) |
| 14 | **Réglages → Données** | `/settings/backups` | Export, restauration, réinitialisation |
| 15 | **Réglages → Customisation** | `/settings/customization` | Thème, son de fin de traitement |
| 16 | **Réglages → Mises à jour** | `/settings/updates` | Version installée, disponibilité, téléchargement |
| 17 | **Réglages → À propos** | `/settings/about` | Identité produit, auteur, revoir la visite guidée |

Plus **deux surfaces transverses** :

- **Tour d'accueil** (onboarding) : 9 étapes au premier lancement, non fermable avant la
  dernière, rejouable depuis « À propos ». Une étape par domaine + ouverture + clôture.
- **En-tête IA global** : sélecteur rapide de fournisseur/modèle, bouton « Tester »
  (benchmark), accès aux réglages IA. Présent sur **tous** les écrans.

Écran de développement seulement (à ne pas redessiner) : une galerie du design system,
retirée du build de production.

---

## 3. Navigation actuelle

```
7 sections → chaque section ouvre un écran par défaut et expose ses onglets
```

| Section | Écrans |
| --- | --- |
| Aujourd'hui | Aujourd'hui |
| Suivi | Candidatures · Calendrier |
| Relations | Entreprises · Réseau |
| Documents | Mes CV · Générer un CV · Mes lettres · Lettre de motivation · Analyser |
| Analyses | Statistiques |
| Profil | Profil |
| Paramètres | IA · Données · Customisation · Mises à jour · À propos |

Faits à conserver, quelle que soit la forme choisie :

- **Aucun raccourci clavier de navigation** n'existe et rien n'en annonce ; en introduire
  est un choix libre de la refonte.
- Un **lien d'évitement** « Aller au contenu » existe et doit survivre.
- La section **Documents porte 5 écrans** : c'est la seule qui a besoin d'une vraie
  navigation interne.
- Les **routes inconnues** retombent sur Aujourd'hui.
- Une **garde de navigation** bloque tout changement d'écran pendant un traitement IA
  actif et demande confirmation avant d'annuler (voir §5).
- Un emplacement d'accessoire dans la barre supérieure existe encore dans le code mais
  **n'est plus utilisé par aucun écran** : les recherches vivent dans les écrans.

---

## 4. Parcours principaux

### P1 — Suivre une candidature (le parcours central)

1. L'utilisateur crée une **entreprise** (ou la choisit si elle existe).
2. Il crée une **candidature** : poste, entreprise, type de contrat, date d'envoi, statut.
3. Il fait avancer le statut : **En attente → Relancée → Entretien → Refusée** (glissé-déposé
   dans le Kanban, menu dans la fiche, ou champ du formulaire).
4. Il programme un **entretien** ou une **relance** rattachés à la candidature.
5. Il retrouve ces échéances dans **Aujourd'hui** et dans le **Calendrier**.
6. Il consulte ses conversions dans **Statistiques**.

### P2 — Constituer son profil

1. **Import IA** : choisir un CV PDF → analyse (Vision ou Texte) → **écran de revue**
   élément par élément (garder / remplacer / ajouter) → application.
2. Ou **saisie manuelle** section par section (10 sections).
3. Le profil calcule une **complétude en %** et liste les sections incomplètes.
4. Le profil alimente ensuite tous les documents.

### P3 — Générer un CV ciblé

1. Coller le **texte de l'offre**.
2. Génération IA (~4 appels, 1 à 2 minutes) avec progression annulable.
3. L'éditeur s'ouvre : **feuille A4 éditable directement** au centre, **aide au contenu** à
   droite, brief à gauche (repliable).
4. L'utilisateur accepte/ignore des recommandations, ajoute des éléments du profil,
   corrige l'orthographe (appel IA explicite), surveille la **place restante**.
5. Il **enregistre** la version (nommée) et/ou **exporte le PDF** (exactement 1 page A4).

### P4 — Rédiger une lettre

1. Brief : entreprise, poste, **ton** (Formel / Naturel / Créatif), **longueur**
   (Courte / Moyenne / Longue), contexte ou offre.
2. Rédaction IA → feuille A4 éditable (colonne d'identité + corps).
3. **Itérations** : consignes successives cumulées (« plus court », puis « plus formel »),
   chaque régénération journalisée avec sa durée.
4. Correction orthographique, enregistrement, export PDF.

### P5 — Analyser un CV existant contre une offre

1. Choisir un PDF (sélecteur natif) + coller l'offre.
2. Analyser → **score ATS** (0–100), récapitulatif, recommandations de reformulation.
3. Les recommandations sont **informatives** ici : elles s'appliquent dans l'éditeur de CV.

### P6 — Configurer l'IA

1. Onglet **IA locale** : catalogue de modèles téléchargeables, gérés par l'application
   (installer / activer / supprimer / tester).
2. Onglet **IA online/personnalisé** : choisir un fournisseur, saisir clé API et endpoint,
   actualiser la liste des modèles, tester la connexion, régler mode et température.
3. **Tester l'IA** lance un benchmark sur un CV de référence embarqué et renvoie un score
   par catégorie.

### P7 — Sauvegarder / restaurer

Export de la base → fichier local. Restauration validée avant remplacement.
Réinitialisation totale (remet aussi le tour d'accueil à « jamais vu »).

---

## 5. Données affichées par écran

Les noms de champs ci-dessous sont ceux du contrat IPC (`snake_case`) : ils sont **figés**,
générés depuis Rust, et ne doivent pas être renommés côté interface.

### 5.1 Aujourd'hui

- Date du jour en clair.
- Bandeau **30 derniers jours** : `applications`, `responses`, `interviews`.
- **Prochainement** : le prochain événement mis en avant + les suivants
  (`upcoming_items[]` : `kind` entretien|relance, `date`, `job_title`, `company_name`,
  `detail`).
- **À faire** : relances en retard (`performance.overdue_follow_ups`), entretiens à
  préparer, relances à envoyer.
- **Candidatures récentes** : `recent[]` (jusqu'aux dernières enregistrées) + « Tout voir ».
- **Activité** : candidatures par semaine (`activity[]` = `start`, `count`).
- **Pipeline** : répartition par statut (`pipeline[]` = `label`, `count`, `percentage`).
- État **entièrement vide** : un écran d'accueil dédié, avec 3 issues (créer une
  candidature, ouvrir les candidatures, ouvrir le calendrier).

### 5.2 Candidatures

Deux vues sur le **même filtre** : **Kanban** (4 colonnes, une par statut, paginées
indépendamment) et **Liste** (table triable, densités 8 / 25 / 50).

Colonnes de la table : Poste (+ domaine professionnel), Entreprise, Ville, Contrat, Durée
hebdomadaire, Type de candidature, Statut, Date d'envoi.

Filtres (tous appliqués **en base**, jamais en mémoire) : recherche libre (poste +
entreprise), statut, type de candidature, code de contrat, domaine professionnel, type
d'entreprise, taille d'entreprise, secteur, régime horaire, bornes d'heures hebdomadaires,
entreprise, ville, intitulé, bornes de date d'envoi. Tri : poste / entreprise / statut / date.

Actions : nouvelle candidature, édition, suppression (simple et multiple par cases à
cocher), changement de statut, **export CSV** (du filtre courant ou de la seule sélection).

Fiche latérale (une candidature) :

- En-tête : poste, entreprise · contrat · ville, sélecteur de statut, Modifier, Supprimer.
- Bloc **Candidature** : type, contrat, durée hebdomadaire, domaine professionnel, envoyée
  le, **ancienneté en jours**, lien de l'offre.
- Bloc **Entreprise** : nom, secteur, type effectif, taille, ville/adresse effectives.
- Contact lié, notes libres.

Règle métier à préserver : **ville, adresse et type d'entreprise sont des surcharges**.
Vide = « hérite de l'entreprise », jamais « non renseigné ». L'interface affiche la valeur
effective (`effective_*`) et peut signaler qu'elle est héritée.

### 5.3 Calendrier

- Vues **Mois** (grille 6×7), **Semaine**, **Jour**.
- Navigation période précédente / suivante / Aujourd'hui, titre de période.
- Compteurs du mois : nombre d'entretiens, nombre de relances.
- Événements : entretiens (avancement) et relances (à traiter), portant heure (entretien
  seulement), libellé = poste, détail = entreprise, icône selon le format ou le canal.
- Clic sur un jour vide → création ; clic sur un événement → édition de son entité.
- Actions d'en-tête : nouvel entretien, nouvelle relance.

### 5.4 Entreprises

- Liste paginée (10 par page) : nom, secteur · ville, type d'entreprise.
- Recherche (nom + ville) et filtres : secteur, type, taille.
- Fiche : nom, **3 métriques** (candidatures, entretiens, en attente), **candidatures
  liées** avec accès à l'écran Candidatures, puis secteur, type, taille, ville, adresse,
  site web, date d'ajout, notes.
- Suppression **refusée** si des candidatures y sont rattachées — le message le dit.

### 5.5 Réseau

- Liste paginée (8 par page) : prénom + nom, poste · entreprise, rôle dans le suivi.
- Recherche et filtre par rôle. Rôles proposés : Recruteur, Manager, Référent, Ancien
  collègue, Autre (champ libre en base).
- Fiche : nom complet, entreprise, poste, e-mail, téléphone, LinkedIn, rôle, notes.
- Suppression refusée si des candidatures ou entretiens le référencent.

### 5.6 Mes CV

- Bibliothèque paginée avec recherche : nom de la version, date de création, score ATS de
  la version sélectionnée.
- Aperçu **A4 fidèle au PDF**.
- Actions sur la version : Modifier (rouvre l'éditeur), Dupliquer, Exporter le PDF,
  Supprimer.
- Deux formats de contenu coexistent : versions récentes (workspace complet, éditables) et
  **versions historiques** (ancien snapshot, encore lisibles, converties à la première
  édition). Les deux doivent rester affichables.

### 5.7 Générer un CV (éditeur)

Trois panneaux : **Offre ciblée** (repliable après génération), **Aperçu A4 éditable**,
**Aide au contenu**.

Aperçu A4 : identité + photo, profil, expériences (puces), projets, groupes de
compétences, formations, certifications, langues. **Édition directe sur le papier.**
Cinq paliers de densité automatiques ; en dépassement, un avertissement et l'export
désactivé.

Aide au contenu :

- **Score ATS** et **indicateur de place** qualitatif (Bonne marge / Espace disponible /
  Peu d'espace restant / CV presque plein / Dépassement) — **jamais un faux pourcentage**.
- **Recommandé pour cette offre** : jusqu'à 4 propositions d'ajout/remplacement, avec
  raison, pertinence qualitative (Très pertinent / Pertinent / Secondaire) et gain de score
  simulé localement. Ajouter / Ignorer.
- **Disponible dans votre profil** : bibliothèque complète des éléments optionnels
  (compétences, projets, certifications, langues) absents du document.
- **Compétences manquantes à vérifier** : exigences de l'offre absentes du profil,
  affichées **sans** bouton d'ajout.
- **Optimisations de rédaction** : reformulations de texte, avec Accepter / Refuser /
  Annuler.

En-tête : durée et tokens de la génération, annuler/rétablir, Corriger l'orthographe,
Enregistrer (nom de version requis), Exporter le PDF.

### 5.8 Mes lettres / Lettre de motivation

- Bibliothèque : entreprise ou nom, poste · ton, date. Actions : Modifier, Copier,
  Exporter le PDF, Supprimer.
- Éditeur : feuille A4 avec **colonne d'identité** (issue du profil, éditable sur la
  feuille — l'enregistrement se fait à la sortie du champ) et **corps** éditable.
- Champs de lettre : entreprise, poste, interlocuteur, adresse destinataire, référence
  d'offre — édités sur la feuille, enregistrés avec la lettre.
- Barre d'outils de mise en forme : **gras, souligné, taille (petite/normale/grande),
  alignement** — et rien d'autre : tout le reste serait perdu à l'export.
- « Pièce jointe : curriculum vitæ » toujours affiché.
- Panneau **Itérations** après la première rédaction : historique des consignes et des
  durées, champ « Que faut-il changer ? », retour au brief possible.
- Quatre paliers de densité ; en dépassement, Exporter et Enregistrer sont désactivés.

### 5.9 Analyse de CV

Zone de dépôt de fichier (PDF, 10 Mo max, sélecteur natif), champ d'offre, bouton
Analyser, Réinitialiser. Résultat : score ATS en pastille, récapitulatif (texte de modèle,
**signalé comme non vérifié**), liste de recommandations par section.

### 5.10 Statistiques

- Période : **30 j / 90 j / Tout**.
- 4 KPI : Candidatures, Entretiens (+ taux), Taux de réponse (+ nombre de réponses), Refus
  reçus (+ % du total).
- **Candidatures envoyées** par semaine (graphique).
- **Funnel de conversion** (étapes, comptes, pourcentages).
- **Candidatures à relancer** : poste, entreprise, date d'envoi, **ancienneté en jours**,
  avec action « relancer » qui ouvre le formulaire de relance.
- **Performance** : délai moyen de réponse, candidatures par semaine, entretiens à venir,
  relances en retard.
- **Export CSV**.

### 5.11 Profil

- En-tête : photo (choisir / retirer), identité, **barre de complétude** avec indice sur
  les sections à compléter, carte de réinitialisation du profil.
- 10 onglets avec compteurs : **Identité, Objectif professionnel, Présence en ligne,
  Expériences, Compétences, Formations, Projets, Certifications, Langues, Centres
  d'intérêts**.
- Identité : prénom, nom, e-mail, téléphone, adresse, ville, date de naissance, âge.
- Objectif : titre, résumé, disponibilité, contrats recherchés.
- En ligne : LinkedIn, GitHub, site.
- Expériences : intitulé, entreprise, lieu, période (ou « Aujourd'hui »), description.
- Compétences : nom + description facultative. Formations : diplôme, école, lieu, période,
  description. Projets : nom, technologies, description, URL. Certifications : nom,
  organisme, date, URL, description. Langues : nom + niveau. Intérêts : nom.
- Chaque section s'édite dans un formulaire dédié ; chaque section vide propose son action.
- **Import depuis un CV** : choix de la méthode (Vision recommandée / Texte), progression,
  puis **écran de revue** listant, par section, chaque élément proposé face à l'existant,
  avec conflit signalé et résolution (garder / remplacer / ajouter comme nouveau) ; enfin
  un récapitulatif (ajoutés / remplacés / ignorés) et la méthode réellement utilisée.
- Réinitialiser le profil n'efface **que** le profil et sa photo.

### 5.12 Réglages → IA

Deux onglets.

**IA locale** — catalogue de modèles gérés par l'application :
catégorie (Très léger / Léger / Équilibré / Puissant / Qualité maximale), éditeur, nom,
description, taille de téléchargement, RAM recommandée, **adéquation à la machine**
(Recommandé / Compatible / Peut être lent / Mémoire insuffisante), installé, actif,
dernier benchmark. Actions : télécharger (avec progression annulable), activer, supprimer,
tester. Plus l'état du moteur local (non installé / téléchargement / extraction / démarrage
/ prêt / erreur), sa version, son port, l'espace disque occupé.

**IA online/personnalisé** — grille de fournisseurs : IA locale Candilog (recommandée),
Mistral, OpenAI, Gemini, Claude, DeepSeek, Personnalisé (compatible OpenAI / Ollama local /
LM Studio) ; Ollama reste supporté pour les configurations existantes.
Configuration : modèle (saisie libre + liste actualisable), endpoint, **clé API** (jamais
affichée en clair, stockée dans le coffre système, supprimable), test de connexion.
Génération : **mode d'analyse** (Auto / Petit / Standard / Avancé) et **température** 0–2.
Chaque fournisseur conserve **sa propre** configuration : basculer n'écrase rien.

**Benchmark** : lancé depuis l'en-tête global, le héros des réglages ou la carte d'un modèle
local. Renvoie un score 0–100, une qualité qualitative, le détail par catégorie, les
métriques de durée (extraction PDF, prétraitement, LLM, parsing), tokens et vitesse,
nombre d'hallucinations, méthode utilisée et repli éventuel. Un fournisseur distant
**avertit** que le CV de référence lui sera envoyé.

### 5.13 Réglages → Données / Customisation / Mises à jour / À propos

- **Données** : créer une sauvegarde, restaurer (confirmation détaillant le repli),
  réinitialiser toutes les données (les référentiels métier survivent).
- **Customisation** : thème (Clair / Sombre / Système), son de fin de traitement
  (Activé / Désactivé, préférence locale à la machine).
- **Mises à jour** : version installée, nouvelle version, état, action (Rechercher /
  Mettre à jour), progression du téléchargement, notes de version. Rien sur le mécanisme.
- **À propos** : logo, nom, version, deux faits (données locales, IA au choix), bouton
  « Revoir la présentation », auteur (Alexandre Bouttier) et accès aux mises à jour.
  **Pas** de hero marketing, **pas** de pile technique exposée.

---

## 6. Interactions importantes

### Traitements IA (transverse, le point le plus délicat)

- **Un seul traitement IA actif** à la fois dans toute l'application.
- Progression : **étape en cours + barre indéterminée + temps écoulé**. Jamais de
  pourcentage — la durée dépend du modèle, un pourcentage serait inventé.
- Les tokens cumulés connus s'affichent pendant, puis **durée totale + tokens** à la fin
  (« Généré en 18,4 s · 1 024 tokens », ou « tokens non communiqués »).
- **Arrêt toujours disponible**, et l'arrêt est immédiat côté backend.
- Un **signal sonore** annonce la fin (préférence désactivable). Une opération annulée
  reste muette.
- **Quitter l'écran pendant un traitement demande confirmation** et annule le traitement.
- Si aucun fournisseur IA n'est configuré, une **modale centrale unique** le dit et renvoie
  aux réglages ; les écrans n'affichent pas de bandeau redondant.

### Listes

- Recherche et filtres sont des **paramètres de requête backend**, jamais un filtrage de la
  page affichée. La recherche est insensible aux accents et à la casse.
- Pagination partout ; le Kanban pagine **chaque colonne indépendamment**.
- Sélection multiple par cases à cocher sur la table des candidatures, avec actions
  groupées (supprimer, exporter).
- Un état vide **avec critères actifs** doit toujours proposer « Tout effacer ».

### Formulaires

- Validation Zod côté interface **et** revalidation Rust : l'interface n'est pas la
  frontière de sécurité. Les erreurs remontent sous le champ concerné.
- Saisie de date au format **JJ-MM-AAAA**, heure **HH:MM**, affichage « 02 août 2026 ».
- Champs obligatoires signalés.
- Règles métier visibles dans les formulaires : le **lien de l'offre est requis** pour une
  candidature en réponse à une offre et **interdit** pour une candidature spontanée ; les
  heures hebdomadaires sont bornées (0 < h ≤ 168).
- Les champs d'offre et de contexte proposent un bouton **« Coller »** (lecture native du
  presse-papiers) en plus de Ctrl+V.

### Destructions

Toute suppression passe par une **confirmation** qui dit ce qui disparaît **et ce qui
survit**. Exemples qui doivent rester exacts : supprimer une candidature supprime ses
entretiens et relances mais conserve l'entreprise et le contact ; supprimer une entreprise
est refusé si des candidatures y sont rattachées ; réinitialiser le profil ne touche à
rien d'autre.

### Documents

- **Édition directe sur la feuille A4** : ce qui est affiché est ce qui sera enregistré et
  exporté. Le collage ne conserve que le texte brut.
- Les aperçus doivent rester **fidèles au PDF** : même géométrie, mêmes polices
  (IBM Plex Sans / Mono embarquées), même logique de densité. C'est la seule contrainte
  visuelle qui survit à la refonte, et elle est fonctionnelle : la feuille reste **blanche
  en thème sombre**, car elle prévisualise une page imprimée, pas une surface d'application.
- Le dépassement de page est **signalé** et bloque l'export ; rien n'est jamais tronqué
  silencieusement.

### Retours et états

Chaque écran doit traiter **chargement**, **erreur avec action de reprise**, **vide avec
issue**. Les écritures sans décision produisent une notification brève ; les décisions
destructives, une confirmation. Aucune information ne doit être portée par la **couleur
seule**.

---

## 7. Contraintes techniques à respecter

### Pile (non substituable)

| Couche | Réalité |
| --- | --- |
| Shell | Tauri 2 (webview native, pas un navigateur) |
| UI | React 19, TypeScript strict, Vite, Tailwind 4 |
| Formulaires | React Hook Form + Zod 4 |
| État serveur | TanStack Query 5 |
| État UI transverse | Zustand — **UI seulement**, jamais les données serveur |
| Graphiques | **Recharts** uniquement |
| Icônes | Material Symbols Rounded, **sous-police locale** limitée à une liste blanche typée |
| Backend | Rust, SQLite via rusqlite |

**Interdits structurels** : shadcn/ui, Radix, une seconde bibliothèque de graphiques, une
seconde bibliothèque de composants, une police d'affichage distante (les polices sont
embarquées, l'application fonctionne hors ligne).

### Architecture à ne pas casser

```
Vue React → ViewModel (hook) → service frontend → ipc() → commande Tauri
                                                              ↓
                                              Service Rust → Repository → SQLite
```

- Le frontend n'appelle **jamais** `invoke` hors du point d'entrée IPC unique.
- Les types TypeScript du contrat IPC sont **générés depuis Rust** et ne s'éditent pas à la
  main. Les noms de champs (`snake_case`) sont figés : la refonte redessine, elle ne
  renomme pas les données.
- Les libellés des référentiels métier viennent de la base par jointure ; l'interface n'en
  tient pas de copie.
- Les composants partagés vivent dans le design system ; les composants métier restent
  dans leur domaine.
- **Aucun prompt IA dans React** : toute l'IA est en Rust.

### Contraintes d'environnement

- **Fenêtre native unique**, pas de responsive mobile. La cible réelle va d'environ
  1280 px à un très grand écran ; l'application doit rester utilisable en fenêtre réduite
  (~1024 px) sans scroll horizontal du corps de page.
- **Pas de route profonde partageable** : c'est une application locale, pas un site. L'URL
  sert d'état interne (un paramètre ouvre la création d'une candidature depuis le tableau
  de bord, un autre porte la fiche sélectionnée).
- Les accès système (dialogues de fichier, presse-papiers, ouverture de liens externes)
  sont **natifs** : la webview n'y accède pas directement. La photo de profil arrive en
  `data:` URL, pas par un chemin de fichier.
- Aucun chemin de fichier ne traverse l'IPC depuis l'interface vers le backend : l'écran
  Analyse ne reçoit que le **nom** du fichier choisi.

### Accessibilité (plancher, à maintenir)

- Un seul `h1` par écran, focus clavier visible partout, lien d'évitement.
- Libellés accessibles sur tout contrôle icône-seule.
- États de chargement, vide et erreur annoncés.
- Aucune donnée accessible par le seul survol : un graphique expose son axe ou une liste
  équivalente.
- Deux séries ou plus dans un graphique → légende systématique avec libellé **et** valeur
  (les teintes de statut vert et rouge sont proches pour une deutéranopie).
- `prefers-reduced-motion` respecté.
- Contraste suffisant en clair **et** en sombre : les deux thèmes sont de premier rang.

---

## 8. Éléments métier qui ne doivent surtout pas disparaître

Liste de contrôle. Si un de ces points n'est plus exprimable après la refonte, la refonte
a supprimé une fonction, pas un habillage.

### Modèle de données

1. **4 statuts de candidature exactement** : En attente, Relancée, Entretien, Refusée.
   Contraints en base ; le Kanban a une colonne par statut.
2. **2 types de candidature** : réponse à une offre (lien obligatoire) ou spontanée (lien
   interdit).
3. **4 référentiels métier distincts, jamais fusionnés** : secteur d'activité *de
   l'entreprise*, domaine professionnel *du poste*, type d'organisation, type de contrat.
   Une banque (secteur) recrute des informaticiens (domaine).
4. **La taille d'entreprise est une 5e dimension indépendante du type** : « ESN + PME » et
   « Association + TPE » doivent rester exprimables. 6 valeurs, dont « non renseigné ».
5. **Ville, adresse et type d'entreprise d'une candidature sont des surcharges** :
   vide = hérité de l'entreprise. Les filtres portent sur la valeur effective.
6. **Régime horaire et volume horaire sont deux champs distincts** : temps plein / partiel /
   non précisé, et un nombre d'heures.
7. Le **profil est la source exhaustive**, le CV en est une **sélection**. Identité,
   coordonnées, expériences et formations forment le socle ; compétences, projets,
   certifications et langues restent en bibliothèque jusqu'à un choix explicite.
8. **Expériences et formations ne sont jamais une sélection** : toutes sont conservées dans
   un CV généré, seuls l'ordre et la mise en avant varient.

### Garanties IA (fragiles, durement acquises)

9. **Le score ATS affiché est un calcul déterministe local**, jamais le chiffre renvoyé par
   le modèle. Idem pour les gains de proposition, simulés localement.
10. Une **offre ou un PDF importé est de la donnée, jamais des instructions** : les contenus
    non fiables sont encadrés côté backend.
11. Les documents générés sont **recadrés sur les faits réels** du profil : rien
    d'inventé n'atteint l'écran.
12. Les **compétences manquantes** de l'offre absentes du profil restent des **écarts
    informatifs** : jamais un bouton qui les ferait passer pour acquises.
13. La **lettre est assemblée, pas rédigée librement** : le modèle ne choisit que des
    identifiants de faits vérifiés et des mots-clés du brief.
14. Les **consignes d'itération se cumulent** : « plus court » puis « plus formel » valent
    ensemble.
15. **Aucun pourcentage de progression** sur un traitement IA ; durée et tokens réels à la
    fin.
16. **Import de CV : Vision par défaut, Texte en repli**, avec le choix exposé, Vision
    désactivée si le modèle ne la supporte pas, et le repli signalé.
17. L'**import de profil passe toujours par une revue** : rien n'est enregistré sans
    validation élément par élément.
18. Le résultat du test de connexion IA est un **message fixe** : la prose du modèle n'est
    pas un état.
19. **Chaque fournisseur garde sa propre configuration** et sa propre clé ; basculer
    n'écrase rien. La clé n'est jamais affichée en clair.
20. L'**IA locale s'appelle « IA locale »** dans l'interface, jamais d'après une famille de
    modèles.

### Garanties produit

21. **Tout reste sur cette machine.** Aucun écran ne doit suggérer un compte, un cloud, une
    synchronisation ou un partage.
22. L'utilisateur n'a pas à connaître la pile technique : ni Tauri, ni React, ni SQLite, ni
    IPC, ni coffre, ni les noms de commandes.
23. **Le PDF exporté fait exactement une page A4**, texte sélectionnable, polices
    embarquées. Un contenu trop long est refusé avec un message qui nomme la vraie cause
    (trop long ≠ trop large) — jamais tronqué en silence.
24. **Export CSV** disponible sur les candidatures et les statistiques.
25. **Sauvegarde / restauration / réinitialisation** restent accessibles, avec leurs
    garanties (validation avant remplacement, copie de secours).
26. Le **tour d'accueil** reste rejouable sans toucher aux données.
27. Les **thèmes clair et sombre** sont tous deux de premier rang, plus le suivi du système.
28. **Aucun texte de modèle n'est présenté comme un fait vérifié** : les sorties libres de
    l'IA sont signalées comme telles.

---

## 9. Ce que la refonte peut librement réinventer

Pour lever toute ambiguïté sur le périmètre de liberté :

- La **forme de la navigation** : rail, barre latérale large, barre supérieure, palette de
  commandes, onglets — au choix, tant que les 17 écrans restent atteignables et que la
  section Documents (5 écrans) reste lisible.
- Le **regroupement des écrans** en sections, et leurs noms.
- La **densité**, l'échelle typographique, les rayons, les ombres, la palette, l'accent.
- Le **découpage spatial** de chaque écran : maître-détail, panneau latéral, plein écran,
  superposition, colonnes.
- La **façon d'afficher** un statut, un score, une progression, une métrique — tant que la
  couleur n'est jamais seule porteuse d'information.
- L'introduction de **raccourcis clavier**, d'une palette de commandes, d'une recherche
  globale : rien n'existe aujourd'hui, tout est permis.
- L'introduction d'**animations** et de transitions, sous réserve de
  `prefers-reduced-motion`.

Ce qui est hors périmètre d'une refonte visuelle : changer le modèle de données, renommer
les champs du contrat IPC, supprimer un écran ou une action, déplacer du métier dans React,
ou substituer un élément de la pile.
