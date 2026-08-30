export const PAGES_LEGALES = [
  { cle: "mentions-legales", libelle: "Mentions légales", href: "/mentions-legales" },
  { cle: "confidentialite", libelle: "Confidentialité", href: "/confidentialite" },
  { cle: "licence", libelle: "Licence", href: "/licence" },
  { cle: "conditions-utilisation", libelle: "Conditions d'utilisation", href: "/conditions-utilisation" },
] as const;

export type ClePageLegale = (typeof PAGES_LEGALES)[number]["cle"];

/* Une date par page : les deux textes n'évoluent pas ensemble, et une constante
   partagée re-daterait à tort la page qu'on n'a pas touchée. */
export const MISE_A_JOUR_CONFIDENTIALITE = "29 août 2026";
export const MISE_A_JOUR_CONDITIONS = "29 août 2026";
