import { GITHUB_REPO } from "./liens";

/** Téléchargements stables : GitHub redirige `…/releases/latest/download/<nom>`
 *  vers l'asset du même nom sur la dernière release publiée. */
const DOWNLOAD = `${GITHUB_REPO}/releases/latest/download`;

export type Plateforme = {
  readonly groupe: string;
  readonly libelle: string;
  readonly extension: string;
  /** `windows` = tuile à quatre carreaux en CSS ; sinon nom du SVG de public/brand. */
  readonly logo: "windows" | "apple" | "ubuntu" | "fedora";
  readonly href: string;
};

export const PLATEFORMES: readonly Plateforme[] = [
  {
    groupe: "Windows",
    libelle: "Windows 10 et 11",
    extension: ".exe",
    logo: "windows",
    href: `${DOWNLOAD}/candilog-windows-latest.exe`,
  },
  {
    groupe: "macOS",
    libelle: "Apple Silicon et Intel",
    extension: ".dmg",
    logo: "apple",
    href: `${DOWNLOAD}/candilog-macos-latest.dmg`,
  },
  {
    groupe: "Linux",
    libelle: "Ubuntu, Debian",
    extension: ".deb",
    logo: "ubuntu",
    href: `${DOWNLOAD}/candilog-ubuntu-latest.deb`,
  },
  {
    groupe: "Linux",
    libelle: "Fedora",
    extension: ".rpm",
    logo: "fedora",
    href: `${DOWNLOAD}/candilog-fedora-latest.rpm`,
  },
];
