import type { MetadataRoute } from "next";

import { SITE_URL } from "@/lib/data/site";

export const dynamic = "force-static";

/* `trailingSlash: true` (next.config.ts) : les pages sont servies avec la barre finale. */
const PAGES = [
  { chemin: "/", priorite: 1 },
  { chemin: "/confidentialite/", priorite: 0.3 },
  { chemin: "/licence/", priorite: 0.3 },
  { chemin: "/conditions-utilisation/", priorite: 0.3 },
  { chemin: "/mentions-legales/", priorite: 0.2 },
] as const;

export default function sitemap(): MetadataRoute.Sitemap {
  return PAGES.map(({ chemin, priorite }) => ({
    url: `${SITE_URL}${chemin}`,
    changeFrequency: "monthly",
    priority: priorite,
  }));
}
