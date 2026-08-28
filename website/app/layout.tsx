import type { Metadata, Viewport } from "next";
import "./globals.css";

const siteUrl = (process.env.NEXT_PUBLIC_SITE_URL ?? "http://localhost:3000").replace(/\/$/, "");
const googleVerification = process.env.NEXT_PUBLIC_GOOGLE_SITE_VERIFICATION;

export const metadata: Metadata = {
  metadataBase: new URL(siteUrl),
  title: {
    default: "Candilog - Votre recherche d'emploi au même endroit",
    template: "%s - Candilog",
  },
  description:
    "Candilog rassemble candidatures, contacts, rendez-vous et documents dans une application de bureau privée pour Windows, macOS et Linux.",
  applicationName: "Candilog",
  category: "productivity",
  alternates: {
    canonical: "/",
    languages: {
      "fr-FR": "/",
    },
  },
  keywords: [
    "suivi candidatures",
    "recherche emploi",
    "CV ATS",
    "lettre de motivation IA",
    "application candidature",
  ],
  authors: [{ name: "Alexandre Bouttier", url: "https://www.alexandrebouttier.fr" }],
  creator: "Alexandre Bouttier",
  publisher: "Alexandre Bouttier",
  formatDetection: {
    email: false,
    address: false,
    telephone: false,
  },
  robots: {
    index: true,
    follow: true,
    googleBot: {
      index: true,
      follow: true,
      "max-image-preview": "large",
      "max-snippet": -1,
      "max-video-preview": -1,
    },
  },
  verification: googleVerification ? { google: googleVerification } : undefined,
  openGraph: {
    type: "website",
    url: "/",
    locale: "fr_FR",
    siteName: "Candilog",
    title: "Candilog - Votre recherche d'emploi au même endroit",
    description:
      "Offres, contacts, documents et rendez-vous réunis dans une application privée sur votre ordinateur.",
    images: [
      {
        url: "/opengraph-image.png",
        width: 1200,
        height: 630,
        alt: "Candilog, l’application de bureau pour organiser une recherche d’emploi",
      },
    ],
  },
  twitter: {
    card: "summary_large_image",
    title: "Candilog - Votre recherche d'emploi au même endroit",
    description:
      "Offres, contacts, documents et rendez-vous réunis dans une application privée sur votre ordinateur.",
    images: [
      {
        url: "/opengraph-image.png",
        width: 1200,
        height: 630,
        alt: "Candilog, l’application de bureau pour organiser une recherche d’emploi",
      },
    ],
  },
  icons: {
    icon: "/brand/candilog.png",
    apple: "/brand/candilog.png",
  },
};

export const viewport: Viewport = {
  themeColor: [
    { media: "(prefers-color-scheme: light)", color: "#eef0f2" },
    { media: "(prefers-color-scheme: dark)", color: "#101114" },
  ],
  colorScheme: "light dark",
  width: "device-width",
  initialScale: 1,
};

export default function RootLayout({ children }: Readonly<{ children: React.ReactNode }>) {
  return (
    <html lang="fr">
      <body>{children}</body>
    </html>
  );
}
