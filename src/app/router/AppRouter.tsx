import { lazy } from "react";
import { createBrowserRouter, RouterProvider, Navigate } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import { AppShell } from "@/app/layout/AppShell";
import { SECTIONS } from "./routes";
import { PlaceholderPage } from "./PlaceholderPage";

const DashboardPage = lazy(() =>
  import("@/features/analyses/view/pages/DashboardPage").then((m) => ({ default: m.DashboardPage })),
);
const AnalysesPage = lazy(() =>
  import("@/features/analyses/view/pages/AnalysesPage").then((m) => ({ default: m.AnalysesPage })),
);
const CandidaturesPage = lazy(() =>
  import("@/features/candidatures/view/pages/CandidaturesPage").then((m) => ({
    default: m.CandidaturesPage,
  })),
);
const CalendrierPage = lazy(() =>
  import("@/features/calendrier/view/pages/CalendrierPage").then((m) => ({
    default: m.CalendrierPage,
  })),
);
const EntreprisesPage = lazy(() =>
  import("@/features/entreprises/view/pages/EntreprisesPage").then((m) => ({
    default: m.EntreprisesPage,
  })),
);
const ReseauPage = lazy(() =>
  import("@/features/contacts/view/pages/ReseauPage").then((m) => ({ default: m.ReseauPage })),
);
const ProfilPage = lazy(() =>
  import("@/features/profil/view/pages/ProfilPage").then((m) => ({ default: m.ProfilPage })),
);
const CvLibraryPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({ default: m.CvLibraryPage })),
);
const CvGeneratorPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({
    default: m.CvGeneratorPage,
  })),
);
const LettersLibraryPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({
    default: m.LettersLibraryPage,
  })),
);
const LetterWriterPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({
    default: m.LetterWriterPage,
  })),
);
const CvAnalysisPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({
    default: m.CvAnalysisPage,
  })),
);
const IaPage = lazy(() =>
  import("@/features/parametres/view/pages/IaPage").then((m) => ({ default: m.IaPage })),
);
const SauvegardesPage = lazy(() =>
  import("@/features/parametres/view/pages/SauvegardesPage").then((m) => ({
    default: m.SauvegardesPage,
  })),
);
const MisesAJourPage = lazy(() =>
  import("@/features/parametres/view/pages/MisesAJourPage").then((m) => ({
    default: m.MisesAJourPage,
  })),
);
const AProposPage = lazy(() =>
  import("@/features/parametres/view/pages/AProposPage").then((m) => ({ default: m.AProposPage })),
);
const DesignGallery = lazy(() =>
  import("@/app/dev/DesignGallery").then((m) => ({ default: m.DesignGallery })),
);

/**
 * Écrans réellement migrés, indexés par chemin.
 *
 * Tous les chemins du rail sont couverts. Un chemin absent retomberait encore sur le jalon
 * « écran non encore migré », conservé si la carte de navigation s'agrandit avant sa page.
 */
const PAGES: Record<string, React.ReactElement> = {
  "/": <DashboardPage />,
  "/analyses": <AnalysesPage />,
  "/suivi/candidatures": <CandidaturesPage />,
  "/suivi/calendrier": <CalendrierPage />,
  "/relations/entreprises": <EntreprisesPage />,
  "/relations/reseau": <ReseauPage />,
  "/profil": <ProfilPage />,
  "/documents/cv": <CvLibraryPage />,
  "/documents/generer-cv": <CvGeneratorPage />,
  "/documents/lettres": <LettersLibraryPage />,
  "/documents/rediger-lettre": <LetterWriterPage />,
  "/documents/analyser": <CvAnalysisPage />,
  "/reglages/ia": <IaPage />,
  "/reglages/sauvegardes": <SauvegardesPage />,
  "/reglages/mises-a-jour": <MisesAJourPage />,
  "/reglages/a-propos": <AProposPage />,
};

/** Chemins dont la page n'est plus un jalon. */
export const CHEMINS_MIGRES = Object.keys(PAGES);

const screenRoutes: RouteObject[] = SECTIONS.flatMap((section) =>
  section.routes.map((route): RouteObject => {
    const element = PAGES[route.path] ?? (
      <PlaceholderPage icon={route.icon} title={route.label} section={section.longLabel} />
    );
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
      { path: "_design", element: <DesignGallery /> },
      { path: "*", element: <Navigate to="/" replace /> },
    ],
  },
]);

export function AppRouter() {
  return <RouterProvider router={router} />;
}
