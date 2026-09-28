import type { Metadata } from "next";

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
import { SITE_DESCRIPTION, SITE_TITRE } from "@/lib/data/site";

export const metadata: Metadata = {
  alternates: { canonical: "/" },
  openGraph: {
    type: "website",
    locale: "fr_FR",
    siteName: "Candilog",
    url: "/",
    title: SITE_TITRE,
    description: SITE_DESCRIPTION,
    images: [{ url: "/og-image.png", width: 1200, height: 630, alt: "Candilog — Suivez chaque candidature. Ciblez chaque document." }],
  },
};

export default function Page() {
  return (
    <>
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
