# Releases natives

Les releases ciblent Linux, Windows et macOS sans Node. Le dépôt public `candilog-releases` garde `latest.json` et les paquets signés. Le client compare SemVer, télécharge dans un fichier temporaire, vérifie la signature minisign avec la clé publique embarquée, installe selon la plateforme puis redémarre.

Une mise à jour dont la signature ne correspond pas doit être supprimée et refusée. Linux distribue AppImage/deb/rpm, Windows un installateur, macOS une archive d'application. Les certificats de signature de code restent distincts de minisign.

