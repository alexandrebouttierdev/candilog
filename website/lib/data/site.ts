import type { Metadata } from "next";

/** Titre et description du site, partagés par les métadonnées, Open Graph et les
 *  données structurées. */
export const SITE_URL = "https://candilog.fr";

export const SITE_TITRE = "Candilog — Suivi de candidatures, CV et lettres ciblés";

export const SITE_DESCRIPTION =
  "Suivi de candidatures, CV et lettres ciblés pour chaque offre. Données sur votre ordinateur, IA au choix. Gratuit, sans compte, Windows, macOS, Linux.";

/* Image de partage statique (public/og-image.png) : un fichier à extension, servi par
   GitHub Pages avec le bon type MIME — une route `opengraph-image` sortirait sans. */
export const OG_IMAGE = {
  url: "/og-image.png",
  width: 1200,
  height: 630,
  alt: "Candilog — Suivez chaque candidature. Ciblez chaque document.",
};

/** Métadonnées propres à une page : titre, description, URL canonique et leurs reprises
 *  Open Graph et Twitter. Next fusionne les métadonnées de façon superficielle : une
 *  page qui déclare `openGraph` remplace tout l'objet du layout, image comprise. */
export function metadonneesPage({
  titre,
  description,
  chemin,
}: {
  titre: string;
  description: string;
  /** Avec la barre finale (`trailingSlash: true`), comme dans `app/sitemap.ts`. */
  chemin: string;
}): Metadata {
  return {
    title: titre,
    description,
    alternates: { canonical: chemin },
    openGraph: {
      type: "website",
      locale: "fr_FR",
      siteName: "Candilog",
      url: chemin,
      title: titre,
      description,
      images: [OG_IMAGE],
    },
    twitter: {
      card: "summary_large_image",
      title: titre,
      description,
      images: [OG_IMAGE.url],
    },
  };
}
