import { GITHUB_REPO } from "./liens";

/* TODO(releases) : remplacer chaque `href` par l'URL de l'artefact de la dernière
   release GitHub — par exemple
   https://github.com/alexandrebouttierdev/candilog/releases/latest/download/Candilog_x64.exe
   En attendant, tout pointe sur la page des releases : le visiteur atterrit au bon
   endroit et rien ne mène à un lien mort. Le site est en export statique (GitHub
   Pages), il n'y a donc pas de route /api/download/[platform] pour rediriger. */
const RELEASES = `${GITHUB_REPO}/releases/latest`;

export type Plateforme = {
  readonly groupe: string;
  readonly libelle: string;
  readonly extension: string;
  /** `windows` = tuile à quatre carreaux en CSS ; sinon nom du SVG de public/brand. */
  readonly logo: "windows" | "apple" | "ubuntu" | "fedora";
  readonly href: string;
};

export const PLATEFORMES: readonly Plateforme[] = [
  { groupe: "Windows", libelle: "Windows 10 et 11", extension: ".exe", logo: "windows", href: RELEASES },
  { groupe: "macOS", libelle: "Apple Silicon et Intel", extension: ".dmg", logo: "apple", href: RELEASES },
  { groupe: "Linux", libelle: "Ubuntu, Debian", extension: ".deb", logo: "ubuntu", href: RELEASES },
  { groupe: "Linux", libelle: "Fedora", extension: ".rpm", logo: "fedora", href: RELEASES },
];
