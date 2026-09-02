# Contribuer à Candilog

Les contributions à Candilog sont les bienvenues. Avant de proposer une modification importante, utilisez les coordonnées publiques du [dépôt officiel](https://github.com/alexandrebouttierdev/candilog) pour vérifier qu'elle correspond au périmètre du projet.

## Préparer une contribution

- Limitez chaque contribution à un objectif clair et évitez les changements sans rapport.
- Respectez l'architecture et les conventions décrites dans [`docs/CODE_RULES.md`](./docs/CODE_RULES.md), [`docs/ARCHITECTURE.md`](./docs/ARCHITECTURE.md) et, pour l'interface, [`docs/DESIGN.md`](./docs/DESIGN.md).
- Installez l'environnement et lancez le projet en suivant [`docs/DEVELOPMENT.md`](./docs/DEVELOPMENT.md).
- Ajoutez ou adaptez les tests utiles au changement proposé.
- Mettez à jour la documentation concernée dans la même contribution lorsque le comportement, la configuration ou une commande change.
- Exécutez les validations applicables documentées dans le [`README.md`](./README.md) : aucune CI ne les rejoue.
- Rédigez les messages de commit en français, avec un préfixe Conventional Commits (`feat:`, `fix:`, `refactor:`, `test:`, `docs:`).
- Décrivez dans la Pull Request le besoin traité, la solution retenue et les contrôles exécutés : le [modèle de Pull Request](./.github/PULL_REQUEST_TEMPLATE.md) en donne la liste, et demande de dire explicitement ce qui **n'a pas** été lancé.

Si vous travaillez avec un agent IA, [`AGENTS.md`](./AGENTS.md) rassemble les règles
absolues du dépôt et les commandes de validation.

## Signaler une anomalie ou une vulnérabilité

Les anomalies et les propositions passent par les [modèles d'issue](https://github.com/alexandrebouttierdev/candilog/issues/new/choose).

Une **faille de sécurité** ne se signale jamais par une issue publique : suivez [`SECURITY.md`](./SECURITY.md), qui décrit le canal privé, le périmètre et les délais visés.

## Licence des contributions

Vous devez avoir le droit de soumettre chaque contribution. Ne copiez pas de code incompatible avec le modèle de licence de Candilog ni de contenu pour lequel vous ne disposez pas des droits nécessaires.

Les contributions doivent être compatibles avec le modèle source-available avec double licence de Candilog. Leur acceptation est conditionnée au mécanisme de contribution et de licence défini par le projet. Un accord contributeur séparé, le [`CLA.md`](./CLA.md), est utilisé lorsque cela est nécessaire afin de préserver la possibilité de proposer Candilog sous la licence non commerciale actuelle et sous une licence commerciale séparée.

Le simple fait d'ouvrir une Pull Request ne transfère pas automatiquement vos droits au mainteneur. Avant d'accepter une contribution externe soumise au CLA, le mainteneur doit définir et appliquer un mode d'acceptation ou de signature explicite. Tant que ce mécanisme n'est pas défini, la contribution peut être examinée, mais elle ne doit pas être fusionnée.
