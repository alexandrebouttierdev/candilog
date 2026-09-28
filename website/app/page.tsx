import type { Metadata } from "next";
import { isValidElement, type ReactNode } from "react";

import { SiteFooter } from "@/components/layout/SiteFooter";
import { SiteHeader } from "@/components/layout/SiteHeader";
import { AiSection } from "@/components/landing/AiSection";
import { DocumentsShowcase } from "@/components/landing/DocumentsShowcase";
import { DownloadCta } from "@/components/landing/DownloadCta";
import { Faq } from "@/components/landing/Faq";
import { Hero } from "@/components/landing/Hero";
import { Privacy } from "@/components/landing/Privacy";
import { ProductTour } from "@/components/landing/ProductTour";
import { Tracking } from "@/components/landing/Tracking";
import { Workflow } from "@/components/landing/Workflow";
import { FAQ } from "@/lib/data/faq";
import { SITE_DESCRIPTION, SITE_TITRE, SITE_URL, metadonneesPage } from "@/lib/data/site";

export const metadata: Metadata = metadonneesPage({
  titre: SITE_TITRE,
  description: SITE_DESCRIPTION,
  chemin: "/",
});

/** Texte brut d'une réponse de FAQ : certaines contiennent un lien, que le JSON-LD ne
 *  peut pas porter. On garde les mots à l'identique, sans le balisage. */
function texte(noeud: ReactNode): string {
  if (typeof noeud === "string" || typeof noeud === "number") return String(noeud);
  if (Array.isArray(noeud)) return noeud.map(texte).join("");
  if (isValidElement<{ children?: ReactNode }>(noeud)) return texte(noeud.props.children);
  return "";
}

/* Données structurées de l'accueil : l'application elle-même et la FAQ affichée plus
   bas, reprise mot pour mot. */
const jsonLd = [
  {
    "@context": "https://schema.org",
    "@type": "SoftwareApplication",
    name: "Candilog",
    applicationCategory: "BusinessApplication",
    operatingSystem: "macOS, Windows, Linux",
    description: SITE_DESCRIPTION,
    url: SITE_URL,
    inLanguage: "fr",
    author: { "@type": "Person", name: "Alexandre Bouttier" },
    offers: { "@type": "Offer", price: "0", priceCurrency: "EUR" },
  },
  {
    "@context": "https://schema.org",
    "@type": "FAQPage",
    inLanguage: "fr",
    mainEntity: FAQ.map(({ question, reponse }) => ({
      "@type": "Question",
      name: question,
      acceptedAnswer: { "@type": "Answer", text: texte(reponse) },
    })),
  },
];

export default function Page() {
  return (
    <>
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{ __html: JSON.stringify(jsonLd).replace(/</g, "\\u003c") }}
      />
      <a
        href="#contenu"
        className="sr-only focus:not-sr-only focus:fixed focus:left-4 focus:top-3 focus:z-50 focus:rounded-r7 focus:bg-ac focus:px-4 focus:py-2 focus:text-on-accent"
      >
        Aller au contenu
      </a>
      <SiteHeader />
      <main id="contenu">
        <Hero />
        <ProductTour />
        <Workflow />
        <Tracking />
        <DocumentsShowcase />
        <AiSection />
        <Privacy />
        <Faq />
        <DownloadCta />
      </main>
      <SiteFooter />
    </>
  );
}
