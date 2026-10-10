# Journal des modifications

Le format suit [Keep a Changelog](https://keepachangelog.com/fr/1.1.0/), et le versionnage
[SemVer](https://semver.org/lang/fr/). Chaque version publiée correspond à un tag
`v<version>` et à une [release GitHub](https://github.com/alexandrebouttierdev/candilog/releases).

## [0.1.0] — 2026-10-10

### Ajouté

- **CV de base** : composer un CV à partir du seul profil, sans offre ni candidature et
  sans IA. La feuille A4 est remplie dès l'ouverture, les sections à faire figurer se
  règlent par interrupteurs, et le document s'enregistre dans la bibliothèque comme les
  autres CV. Rouvrir un CV de base affiche ce qui a été enregistré, retouches comprises.

### Modifié

- Thème clair : fonds, filets et encres passent du sable au gris froid, la même famille
  que le thème sombre. L'indigo de la marque et la sélection ne sont plus les seules notes
  froides posées sur du chaud. Le texte secondaire repasse au-dessus du seuil de contraste
  AA, et les filets de séparation redeviennent visibles.
- Barre de titre et en-tête des surcouches : 64 px au lieu de 40, pour que le fil d'Ariane
  et les actions respirent.
- La croix de fermeture des surcouches s'aligne sur le fil d'Ariane des écrans et porte une
  teinte qui la rend repérable : c'est la seule sortie d'une surface plein écran.

### Corrigé

- Le `README` affirmait que Candilog ne cherche jamais de mise à jour tout seul, et que ses
  accès réseau se font tous à la demande. L'application interroge l'API GitHub au démarrage,
  une fois par jour au plus, sans rien envoyer d'autre que cette question.

## [0.0.4] — 2026-10-09

### Corrigé

- IA locale : l'empreinte attendue du moteur Ollama macOS est celle du fichier actuel,
  re-téléversé par Ollama le 16 décembre 2025 ; l'installation ne refuse plus le
  téléchargement sur Mac.
- Analyse face à l'offre : l'état vide « Prêt à analyser » est centré dans le panneau,
  comme les états neutres des autres générateurs.

## [0.0.3] — 2026-10-08

### Ajouté

- Signature de code macOS : le binaire est signé avec un certificat *Developer ID
  Application* et notarié par Apple. Gatekeeper ouvre Candilog sans intervention.

## [0.0.2] — 2026-10-08

### Modifié

- Republication des paquets 0.0.1 : la release GitHub a été supprimée par erreur, et un
  tag déjà posé n'est pas republié par le workflow. Aucun changement de code ni de données.

## [0.0.1] — 2026-10-06

Première version publique.

### Ajouté

- Suivi des candidatures en kanban ou en table, filtres et recherche exécutés en base,
  historique de statut, export CSV.
- Répertoire d'entreprises et de contacts, avec héritage des valeurs de l'entreprise
  (ville, adresse, type) sur la candidature.
- Calendrier des entretiens et des relances.
- Profil professionnel, génération de CV et de lettres de motivation en PDF A4 d'une page,
  analyse ATS déterministe.
- IA locale embarquée : moteur d'inférence llama.cpp lié au binaire, quatre profils de
  modèles GGUF installables depuis les réglages, téléchargés depuis Hugging Face et vérifiés
  par empreinte SHA-256 avant usage. Aucun serveur ni outil externe à installer.
- Autres fournisseurs IA au choix : Ollama (local), Claude, OpenAI, Gemini, Mistral,
  DeepSeek ou point de terminaison personnalisé. La clé API vit dans le coffre du système.
- Sauvegarde et restauration de la base, avec retour arrière en cas d'échec.
- Mise à jour assistée depuis les GitHub Releases : empreinte SHA-256 vérifiée avant
  l'ouverture de l'installateur, jamais d'installation silencieuse.
- Attestation de provenance Sigstore sur chaque binaire publié, vérifiable par
  `gh attestation verify`.
- Licences des composants redistribués (polices IBM Plex, Material Symbols, llama.cpp)
  livrées avec chaque paquet, sous `licenses/`.

### Modifié

- Refonte v2 (en cours) — nouvelle coque : barre de titre avec fil d'Ariane et onglets de
  vue, navigation à six destinations (Aujourd'hui, Candidatures, Relations, Documents,
  Intelligence artificielle, Profil) avec décomptes, barre d'état rappelant le contrat
  clavier de l'écran. Les Réglages deviennent une surcouche (`⌘,`) en six sections ;
  Apparence regroupe thème, densité des listes, animations et son de fin de traitement.
- Refonte v2 — palette de commandes `⌘K` et navigation au clavier (`G` puis `A/C/R/D/P`).
- Refonte v2 — nouvelle palette de couleurs, titres en IBM Plex Serif, données en IBM Plex
  Mono ; dialogues à trois registres qui énumèrent les conséquences ; une seule notification
  à la fois ; fenêtre utilisable dès 940 × 560 px, navigation réduite sous 1060 px.

- Refonte v2 — Candidatures : chaque candidature reçoit une référence lisible (`CAN-142`) et
  un canal « Trouvée via » (Offre, Site, Réseau, Spontanée) ; les candidatures existantes
  sont numérotées dans leur ordre de création. La vue Liste devient une liste groupée par
  statut (Refusée repliée), avec échéance en pastille (entretien, relance, silence de plus
  de 14 jours) et colonnes selon la largeur disponible. Nouvel inspecteur avec historique
  des statuts, menu d'actions (clic droit ou `⋯`), dialogue de suppression qui énumère
  relances, entretiens et historique emportés, duplication (`⌘D`), programmation d'une
  relance (`R`), changement de statut (`S`). Le formulaire garde toutes les précisions dans
  « Plus de détails » et demande confirmation avant de fermer une fiche modifiée.
- Refonte v2 — Aujourd'hui : les échéances en trois horizons (en retard, aujourd'hui, cette
  semaine) ; une relance se marque faite (`⏎`) ou se reporte (`R`), et reste dans
  l'historique. Colonne Situation : répartition des statuts, 30 derniers jours,
  candidatures sans réponse. Le décompte de la navigation compte les retards et le jour.
- Refonte v2 — Candidatures : barre de filtres à puces (« Contrat est CDI, CDD ») ; un clic
  sur une puce inverse la condition (« n'est pas »). Menu « + Filtre » (`F`) à deux niveaux
  couvrant tous les critères, canal et entreprise compris ; recherche sur `/`. Barre
  d'actions groupées sur les cases cochées : statut, export CSV, suppression.
- Refonte v2 — Kanban : colonnes élastiques, cartes compactes avec échéance, bandeaux
  « sans réponse » et « entretien aujourd'hui » ; toast `CAN-142 → Entretien`.
- Les raccourcis à une lettre restent actifs quand le focus est sur une case à cocher.
- Refonte v2 — Relations : entreprises et contacts réunis sur un écran à bascule. Les
  entreprises sont groupées en cours / repérées / clôturées, les contacts en recruteurs et
  managers / réseau ; inspecteur avec candidatures rattachées et historique. « Nouvelle
  candidature » depuis une entreprise la préremplit.
- Refonte v2 — Documents : CV et lettres réunis dans une bibliothèque à onglets (Tous, CV,
  Lettres, Analyses), score ATS affiché dans la liste et l'inspecteur ; Ouvrir, PDF,
  dupliquer, copier et supprimer depuis l'inspecteur ou le clic droit.
- Refonte v2 — Profil : colonne des sections avec leur état et leur décompte, contenu de la
  section à droite ; retrait d'une entrée en un clic (confirmé), import de CV et
  réinitialisation sous les sections.
- IA — routage par tâche : chaque tâche (CV ciblé, lettre, analyse de CV, extraction
  d'offre, lecture d'un CV importé) peut utiliser son propre fournisseur et son propre
  modèle. Sans choix, elle suit le fournisseur principal ; une tâche mal configurée
  s'arrête avec un message au lieu de basculer ailleurs.
- Refonte v2 — Intelligence artificielle : colonne des fournisseurs avec leur état, détail
  factuel (confidentialité, coût, hors connexion) et section « Qui fait quoi » pour choisir
  le modèle de chaque tâche, enregistré aussitôt.
- Refonte v2 — Générateurs de CV et de lettre en surcouche plein écran : offre visée prise
  dans une candidature ou collée, feuille A4 au centre, étapes avec leur durée mesurée à
  droite ; corrections de la lettre sous la feuille, avec consignes rapides.
- Refonte v2 — Vues enregistrées : un filtre de Candidatures s'enregistre sous un nom et
  apparaît dans la section « Vues » de la navigation, avec son décompte. Une vue se met à
  jour, se renomme, se duplique ou se supprime.
- Candidatures : un filtre sur le statut ne laisse plus la liste en chargement, et
  « Statut n'est pas … » affiche les autres groupes au lieu du statut écarté.
- Refonte v2 — Installer l'IA locale : surcouche plein écran pour choisir le modèle, suivre
  le moteur puis le modèle (débit, temps restant) et vérifier par une phrase de test dont la
  durée de réponse est affichée. Les modèles installés s'affichent en lignes dans
  Intelligence artificielle.
- Refonte v2 — Générateurs : colonne « En cours » avec barre d'avancement, temps écoulé et
  tokens ; « Arrêter » (`⌘.`) demande confirmation et laisse finir si l'on renonce ;
  l'offre visée s'ouvre sur les candidatures ; la lettre part d'une feuille neutre, avec
  « Écrire la lettre moi-même ».
- Générateurs : la première étape n'est plus perdue quand le backend l'annonce avant que
  l'écran l'écoute, et les étapes restent dans leur ordre.
- Générateurs : « Ce que l'IA peut utiliser » (CV) et « Arguments autorisés » (lettre)
  retirent des sections du profil avant tout envoi au modèle ; le CV se rédige sur un ton
  sobre, professionnel ou direct ; la lettre peut s'appuyer sur la disponibilité déclarée
  dans le profil.
- Lettre : score d'adéquation à l'offre (part des exigences abordées) et recommandations
  tirées de votre profil, à appliquer ou ignorer ; les exigences que le profil ne prouve pas
  sont signalées, jamais inventées.
- IA — confirmation du premier envoi à un service distant : avant qu'une tâche parte vers
  un service hors de l'ordinateur, Candilog dit qui reçoit quoi ; « Annuler » n'envoie rien.
  « Ne plus demander » se règle par service et se remet à zéro dans Réglages. Un Ollama ou un
  service personnalisé sur une autre machine compte comme distant, y compris dans « Qui
  fait quoi ».
- Candidatures — « Grouper : statut ▾ » : la liste se groupe aussi par entreprise ou par
  contrat, avec des décomptes exacts sur tout le filtre.
- Relations — export CSV : entreprises et contacts dans deux fichiers côte à côte, 9 colonnes
  chacun (état de la relation, candidatures, dernière interaction), lisibles tels quels dans
  un tableur.
- Analyse — « Rythme d'envoi » est dessiné avec les primitives du design ; la bibliothèque
  Recharts est retirée.
- Documents — versions : enregistrer un CV ou une lettre rouverts depuis la bibliothèque
  ajoute une version au lieu d'un nouveau document. L'inspecteur liste les versions (v1,
  v2…) et « Revenir à la version » rend l'une d'elles courante sans rien effacer. Supprimer
  un document supprime ses versions. Chaque document existant devient sa v1 (migration 8,
  additive).
- Relations — historique complet dans la fiche d'une entreprise ou d'un contact :
  candidatures envoyées, changements de statut, entretiens, relances faites et ajout de la
  fiche, du plus récent au plus ancien. « Note » y ajoute un fait daté (un appel, une
  réponse), supprimable ; les notes disparaissent avec leur fiche. Migration 7, additive.
- Refonte v2 — la police d'icônes Material Symbols est retirée : icônes au trait dessinées
  pour Candilog, glyphes typographiques des maquettes (`‹ › ▾ ✕ ✓`) et glyphes de statut.
  Le paquet perd la sous-police (~130 Kio) et sa licence Apache 2.0.
- Refonte v2 — Analyse de CV face à une offre : chaque exigence de l'offre est listée,
  couverte, partielle ou absente, avec la preuve trouvée dans le CV ; score et détail à
  gauche.
- Refonte v2 — Calendrier : barre d'outils compacte, grille plate, pastilles par
  entreprise ; les relances en retard ressortent en rouge et les relances faites
  s'atténuent.
- Refonte v2 — Analyse : parcours des candidatures, taux de réponse par canal « Trouvée
  via » et constats tirés des chiffres de la période.
- Refonte v2 — Réglages : la surcouche occupe toute la fenêtre ; Données, Mises à jour et
  À propos passent à la présentation en lignes des autres sections.
- Palette `⌘K` : chaque action IA indique le modèle qui fera la tâche et s'il tourne sur
  l'ordinateur ou part vers un service distant ; le pied de navigation résume le routage
  (« Routage IA · 3 loc · 1 dist ») dès qu'une tâche a son propre modèle.
- Refonte v2 — Import de CV en surcouche plein écran, la revue élément par élément gagnant
  toute la largeur.
- Export CSV des candidatures : colonnes `reference` et `canal` ajoutées.
- Le lien de l'offre n'est plus exigé que pour une offre publiée ; il devient facultatif
  pour une candidature trouvée sur le site de l'entreprise ou par le réseau.

- Lettres de motivation : rédaction LLM à partir d’un pack d’évidences courtes (plus de collage template du CV), nettoyage d’offre (codes REC, slogans, process RH), grounding factuel Rust (zéro invention). Fallback template si brouillon trop court.

- Score ATS : reclassement automatique des exigences soft (Agile, qualité, MOA, UX, IA…) hors compétences dures pour éviter la dilution (~31/100) ; famille Java/Spring pour crédit partiel.

- Score ATS : soft skills / mots-clés / proximité métier en **bonus** (plus en moyenne avec des 0 qui écrasaient vers ~17/100) ; équivalence CI/CD ↔ GitLab CI / intégration continue ; années aussi déduites des plages de dates.

- Score ATS multi-métiers : matching exact / transférable (familles d'outils génériques),
  corpus candidat élargi (projets, certifications, langues), proximité métier en bonus,
  mots-clés dédupliqués des compétences, prompts offre / recommandations plus stricts
  (exigences vs contexte entreprise, pas d'invention). Tests Open + multi-domaines.

- Score ATS : exigences structurées par catégorie, importance et caractère obligatoire ;
  pondération dynamique, détail explicable par catégorie, correspondances exactes,
  équivalentes, transférables ou partielles, et plafonnement des qualifications
  réglementaires obligatoires absentes. Le même calcul suit les extractions Vision et Texte.

- Analyse de CV : recommandations reliées à une exigence et à des preuves du CV ; les
  reformulations répétitives ou ajoutant une exigence absente sont écartées, et les écarts
  restent informatifs sans action d'ajout.

- Analyse de CV : score déterministe enrichi avec le texte PDF brut (plus seulement le JSON LLM) ; mode Vision si le modèle le permet, avec repli Texte.

- Analyse de CV : ne plus scorer l'expérience à 0 quand les dates manquent ;
  déduction du titre et des années depuis le texte ; filtre des compétences
  issues du blurb entreprise (ex. « Expertises reconnues ») hors exigences poste.

- Analyse de CV : padding horizontal autour du nom de fichier ; marge sous « Offre ciblée ».

- Import CV : descriptions de compétences et de projets mieux récupérées (clés
  alternatives `detail` / `resume`, projets en chaîne, séparation d'une description
  collée dans le libellé avant le recadrage sur le texte du CV).

- Réglages IA / Configuration : liste de modèles compacte avec logo du fournisseur ;
  bouton « Tester » (benchmark) à côté de « Tester la connexion », désactivé sans modèle.

- Réglages IA : le bandeau d'état (fournisseur / modèle / Configuré) en tête de page
  est retiré. « Tester la connexion » est aligné à droite du titre de la carte
  Configuration.

- Top bar : conserve le sélecteur de modèle, « Tester » et le raccourci réglages IA ;
  les notes / recherches / autres actions contextuelles (Profil, Analyses, Documents,
  Aujourd'hui, Réglages, Calendrier) quittent le bandeau. Mes lettres a une recherche
  in-page alignée sur Mes CV. Les onglets Profil ne forcent plus le scroll horizontal
  de la page.

- Import de profil depuis un CV : dates de fin découpées depuis une plage collée dans
  `start_date`, descriptions d'expériences / projets / certifications mieux conservées
  (recadrage par fragments ; en Vision, les textes libres et le prénom / nom lus sur les
  images ne sont plus effacés par un PDF complémentaire partiel). Un nom complet coincé
  dans un seul champ est découpé. Compétences et certifications acceptent une description
  facultative. Mesure locale : voir `src-tauri/examples/cv_import_baseline.md`
  et `src-tauri/examples/PROMPT_IMPORT_CV.md`.

- Page Profil : sections Identité, Objectif professionnel et Présence en ligne en onglets
  dédiés (modales séparées). Le bouton « Modifier le profil » et le panneau latéral
  Identité sont retirés. Le bloc « Réinitialiser le profil » est aligné en haut à droite
  du bandeau, visible quel que soit l'onglet.

- Benchmark « Tester l'IA » : le score Formations accepte les libellés paraphrasés ou
  partiels (diplôme / école), aligné sur ce que l'import affiche déjà.

- Fournisseur IA NVIDIA remplacé par DeepSeek (API compatible OpenAI,
  `https://api.deepseek.com`, modèle par défaut `deepseek-v4-flash`). Un réglage encore
  enregistré sous `nvidia` est relu comme DeepSeek.

### Retiré

- Composants de la v1 que plus aucun écran n'affichait : tableau, tiroir de détail, barre de
  filtres, liste maître, en-tête de fiche, chronologie, surface vitrée, sélecteur rapide
  d'IA. La planche de vérification `/_design` montre désormais les primitives v2.

- Tri par colonne de la table des candidatures : la liste v2 est groupée par statut, les
  plus récentes d'abord.
- Tour d'accueil : le premier lancement ouvre directement Aujourd'hui, dont l'état vide
  propose les premières actions.
