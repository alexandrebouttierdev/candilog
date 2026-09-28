/** Sections de la landing, dans l'ordre de la page : en-tête, menu mobile, 404. */
export const NAV_SECTIONS = [
  { libelle: "Produit", href: "#produit" },
  { libelle: "Parcours", href: "#parcours" },
  { libelle: "Suivi", href: "#suivi" },
  { libelle: "Documents", href: "#documents" },
  { libelle: "IA", href: "#ia" },
  { libelle: "Confidentialité", href: "#confidentialite" },
  { libelle: "FAQ", href: "#faq" },
] as const;

export const NAV_LEGALE = [
  { libelle: "Mentions légales", href: "/mentions-legales" },
  { libelle: "Confidentialité", href: "/confidentialite" },
  { libelle: "Licence", href: "/licence" },
  { libelle: "Conditions d'utilisation", href: "/conditions-utilisation" },
] as const;
