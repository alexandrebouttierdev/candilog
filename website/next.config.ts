import type { NextConfig } from "next";

const nextConfig: NextConfig = {
  // Un yarn.lock traîne dans /home/alex : sans cette racine explicite, Turbopack
  // remonte jusqu'au dossier personnel pour deviner la racine du projet.
  turbopack: { root: import.meta.dirname },
  /* Le site est hébergé sur GitHub Pages (cf. mentions légales) : export statique.
     Conséquence assumée — pas de route serveur, donc pas de /api/download/[platform] :
     les boutons de téléchargement pointent directement sur les releases GitHub. */
  output: "export",
  images: { unoptimized: true },
  // Une page par dossier (/confidentialite/index.html) : GitHub Pages sert les
  // URL sans extension telles quelles.
  trailingSlash: true,
  // Domaine personnalisé candilog.fr (public/CNAME) : le site est servi à la racine,
  // donc ni basePath ni assetPrefix.
};

export default nextConfig;
