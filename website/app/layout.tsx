import type { Metadata, Viewport } from "next";
import { IBM_Plex_Mono, IBM_Plex_Sans, IBM_Plex_Serif } from "next/font/google";

import {
  OG_IMAGE,
  SITE_DESCRIPTION as DESCRIPTION,
  SITE_TITRE as TITRE,
  SITE_URL,
} from "@/lib/data/site";

import "./globals.css";

/* Les trois familles du design system v2 de l'application : Plex Serif pour les titres,
   Plex Mono pour les références et les dates, Plex Sans pour la feuille A4 (identique
   au PDF exporté). `next/font` les télécharge au build et les sert depuis le site :
   aucune requête vers Google au chargement d'une page. */
const plexSerif = IBM_Plex_Serif({
  subsets: ["latin"],
  weight: ["500", "600"],
  variable: "--font-plex-serif",
  display: "swap",
});

const plexMono = IBM_Plex_Mono({
  subsets: ["latin"],
  weight: ["400", "500"],
  variable: "--font-plex-mono",
  display: "swap",
});

const plexSans = IBM_Plex_Sans({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-plex-sans",
  display: "swap",
});

export const metadata: Metadata = {
  metadataBase: new URL(SITE_URL),
  title: TITRE,
  description: DESCRIPTION,
  applicationName: "Candilog",
  authors: [{ name: "Alexandre Bouttier", url: "https://www.alexandrebouttier.fr" }],
  keywords: [
    "suivi de candidatures",
    "recherche d'emploi",
    "CV ciblé",
    "lettre de motivation",
    "analyse ATS",
    "kanban candidatures",
    "IA locale",
    "application de bureau",
  ],
  openGraph: {
    type: "website",
    locale: "fr_FR",
    siteName: "Candilog",
    title: TITRE,
    description: DESCRIPTION,
    images: [OG_IMAGE],
  },
  twitter: {
    card: "summary_large_image",
    title: TITRE,
    description: DESCRIPTION,
    images: [OG_IMAGE.url],
  },
  robots: { index: true, follow: true },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#f1f0ec" },
    { media: "(prefers-color-scheme: dark)", color: "#101114" },
  ],
};

/* Anti-flash : applique data-theme avant le premier paint.
   Sans ce script, un visiteur en mode sombre voit un flash clair. */
const themeScript = `(function(){try{var k="candilog-theme",v=localStorage.getItem(k);
if(v!=="dark"&&v!=="light"){v=window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";}
document.documentElement.setAttribute("data-theme",v);}catch(e){}})();`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html
      lang="fr"
      data-theme="light"
      suppressHydrationWarning
      className={`${plexSerif.variable} ${plexMono.variable} ${plexSans.variable}`}
    >
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body>{children}</body>
    </html>
  );
}
