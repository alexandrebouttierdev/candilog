import type { Metadata } from "next";
import { JetBrains_Mono } from "next/font/google";

import "./globals.css";

const jetbrains = JetBrains_Mono({
  subsets: ["latin"],
  weight: ["400", "500", "600"],
  variable: "--font-jetbrains",
  display: "swap",
});

export const metadata: Metadata = {
  title: "Candilog — Organisez votre recherche d'emploi",
  description:
    "Application desktop pour suivre vos candidatures, vos documents et vos entretiens. Windows, macOS, Linux.",
};

/* Anti-flash : applique data-theme avant le premier paint.
   Sans ce script, un visiteur en mode sombre voit un flash blanc. */
const themeScript = `(function(){try{var k="candilog-theme",v=localStorage.getItem(k);
if(v!=="dark"&&v!=="light"){v=window.matchMedia("(prefers-color-scheme: dark)").matches?"dark":"light";}
document.documentElement.setAttribute("data-theme",v);}catch(e){}})();`;

export default function RootLayout({ children }: { children: React.ReactNode }) {
  return (
    <html lang="fr" data-theme="light" suppressHydrationWarning>
      <head>
        <script dangerouslySetInnerHTML={{ __html: themeScript }} />
      </head>
      <body className={jetbrains.variable}>{children}</body>
    </html>
  );
}
