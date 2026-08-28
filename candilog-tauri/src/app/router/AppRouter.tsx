import { createBrowserRouter, RouterProvider, Navigate } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import { AppShell } from "@/app/layout/AppShell";
import { SECTIONS } from "./routes";
import { PlaceholderPage } from "./PlaceholderPage";
import { DesignGallery } from "@/app/dev/DesignGallery";
import { CalendrierPage } from "@/features/calendrier";
import { CandidaturesPage } from "@/features/candidatures";
import { EntreprisesPage } from "@/features/entreprises";
import { ReseauPage } from "@/features/contacts";
import { AnalysesPage, DashboardPage } from "@/features/analyses";
import { ProfilPage } from "@/features/profil";

/**
 * Écrans réellement migrés, indexés par chemin.
 *
 * Chaque tranche de migration ajoute ici la page de sa feature ; les chemins absents de
 * cette table retombent sur le jalon « écran non encore migré », ce qui garde la navigation
 * complète et rend visible ce qui reste à faire.
 */
const PAGES: Record<string, React.ReactElement> = {
  "/": <DashboardPage />,
  "/analyses": <AnalysesPage />,
  "/suivi/candidatures": <CandidaturesPage />,
  "/suivi/calendrier": <CalendrierPage />,
  "/relations/entreprises": <EntreprisesPage />,
  "/relations/reseau": <ReseauPage />,
  "/profil": <ProfilPage />,
};

const screenRoutes: RouteObject[] = SECTIONS.flatMap((section) =>
  section.routes.map((route): RouteObject => {
    const element = PAGES[route.path] ?? (
      <PlaceholderPage icon={route.icon} title={route.label} section={section.longLabel} />
    );
    // La racine est déclarée en route index et non en chemin vide : React Router refuse
    // de porter les deux à la fois, et `exactOptionalPropertyTypes` interdit de passer
    // `path: undefined` pour contourner la distinction.
    return route.path === "/"
      ? { index: true, element }
      : { path: route.path.slice(1), element };
  }),
);

const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [
      ...screenRoutes,
      // Planche de revue du design system : atteignable par l'URL, jamais par le rail.
      { path: "_design", element: <DesignGallery /> },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);

export function AppRouter() {
  return <RouterProvider router={router} />;
}
