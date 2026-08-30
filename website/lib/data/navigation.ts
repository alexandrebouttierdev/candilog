import { GITHUB_REPO } from "./liens";

export const NAV_SECTIONS = [
  { libelle: "Parcours", href: "#parcours" },
  { libelle: "Suivi", href: "#suivi" },
  { libelle: "CV et analyse", href: "#cv" },
  { libelle: "IA", href: "#ia" },
  { libelle: "Code source", href: "#opensource" },
  { libelle: "FAQ", href: "#faq" },
] as const;

/** Le pied de page reprend la nav mais remplace « Code source » par
 *  « Télécharger » puis un lien GitHub sortant. */
export const NAV_PIED = [
  { libelle: "Parcours", href: "#parcours" },
  { libelle: "Suivi", href: "#suivi" },
  { libelle: "CV et analyse", href: "#cv" },
  { libelle: "IA", href: "#ia" },
  { libelle: "FAQ", href: "#faq" },
  { libelle: "Télécharger", href: "#telecharger" },
] as const;

export const LIEN_GITHUB_PIED = { libelle: "GitHub", href: GITHUB_REPO } as const;

export const NAV_LEGALE = [
  { libelle: "Mentions légales", href: "/mentions-legales" },
  { libelle: "Confidentialité", href: "/confidentialite" },
  { libelle: "Licence", href: "/licence" },
  { libelle: "Conditions d'utilisation", href: "/conditions-utilisation" },
] as const;
