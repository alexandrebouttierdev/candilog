import { createBrowserRouter, RouterProvider, Navigate } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import { AppShell } from "@/app/layout/AppShell";
import { SECTIONS } from "./routes";
import { PlaceholderPage } from "./PlaceholderPage";
import { DesignGallery } from "@/app/dev/DesignGallery";

/**
 * Écrans de l'application, dérivés de la carte de navigation.
 *
 * Les écrans sont pour l'instant des jalons : chaque tranche de migration remplace celui de
 * sa feature par la vraie page (cf. `docs/migration/01-AUDIT.md`, §7). Déclarer les routes
 * dès maintenant fige la carte de navigation et rend la coque vérifiable.
 */
const screenRoutes: RouteObject[] = SECTIONS.flatMap((section) =>
  section.routes.map((route): RouteObject => {
    const element = (
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
