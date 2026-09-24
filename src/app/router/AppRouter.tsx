import { createElement, lazy, Suspense, useEffect } from "react";
import { createBrowserRouter, Navigate, RouterProvider, useLocation, useParams } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import { AppShell } from "@/app/layout/AppShell";
import { useUiStore } from "@/shared/lib/ui-store";
import type { SettingsSection } from "@/shared/lib/ui-store";
import { LEGACY_REDIRECTS } from "./routes";

const TodayPage = lazy(() =>
  import("@/features/analytics/view/pages/TodayPage").then((m) => ({ default: m.TodayPage })),
);
const AnalyticsPage = lazy(() =>
  import("@/features/analytics/view/pages/AnalyticsPage").then((m) => ({ default: m.AnalyticsPage })),
);
const ApplicationsRoute = lazy(() =>
  import("./ApplicationsRoute").then((m) => ({ default: m.ApplicationsRoute })),
);
const CalendarPage = lazy(() =>
  import("@/features/calendar/view/pages/CalendarPage").then((m) => ({ default: m.CalendarPage })),
);
const RelationsPage = lazy(() =>
  import("@/features/relations/view/pages/RelationsPage").then((m) => ({ default: m.RelationsPage })),
);
const ProfilePage = lazy(() =>
  import("@/features/profile/view/pages/ProfilePage").then((m) => ({ default: m.ProfilePage })),
);
const DocumentsPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({ default: m.DocumentsPage })),
);
const ResumeGeneratorPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({ default: m.ResumeGeneratorPage })),
);
const LetterWriterPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({ default: m.LetterWriterPage })),
);
const ResumeAnalysisPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({ default: m.ResumeAnalysisPage })),
);
const AiPage = lazy(() =>
  import("@/features/settings/view/pages/AiPage").then((m) => ({ default: m.AiPage })),
);

/**
 * Atelier du design system, réservé au développement.
 *
 * `import.meta.env.DEV` est une constante à la compilation : le build de production
 * élimine la branche, et avec elle l'import dynamique — la galerie ne se retrouve donc pas
 * dans le paquet distribué, où un écran d'atelier n'a rien à faire.
 */
const designRoutes: RouteObject[] = import.meta.env.DEV
  ? [
      {
        path: "_design",
        element: (
          <Suspense fallback={null}>
            {createElement(
              lazy(() => import("@/app/dev/DesignGallery").then((m) => ({ default: m.DesignGallery }))),
            )}
          </Suspense>
        ),
      },
    ]
  : [];

/** Redirige un ancien chemin v1 vers son équivalent v2, en conservant la requête (`?id=`). */
function LegacyRedirect({ to }: { to: string }) {
  const { search } = useLocation();
  return <Navigate to={`${to}${search}`} replace />;
}

const SECTIONS_V1: Readonly<Record<string, SettingsSection>> = {
  backups: "data",
  customization: "appearance",
  updates: "updates",
  about: "about",
};

/** Ancien écran de réglages v1 : ouvre la surcouche sur sa section, au-dessus d'Aujourd'hui. */
function LegacySettings() {
  const { section = "" } = useParams();
  const openSettings = useUiStore((state) => state.openSettings);
  useEffect(() => {
    openSettings(SECTIONS_V1[section] ?? "appearance");
  }, [openSettings, section]);
  return <Navigate to="/" replace />;
}

/** Écrans de la v2, indexés par chemin (sans la barre oblique initiale). */
export const ROUTES: RouteObject[] = [
  { index: true, element: <TodayPage /> },
  { path: "applications", element: <ApplicationsRoute view="list" /> },
  { path: "applications/kanban", element: <ApplicationsRoute view="kanban" /> },
  { path: "applications/calendar", element: <CalendarPage /> },
  { path: "applications/analytics", element: <AnalyticsPage /> },
  { path: "relations/companies", element: <RelationsPage kind="companies" /> },
  { path: "relations/contacts", element: <RelationsPage kind="contacts" /> },
  { path: "documents", element: <DocumentsPage filter="all" /> },
  { path: "documents/resumes", element: <DocumentsPage filter="resumes" /> },
  { path: "documents/letters", element: <DocumentsPage filter="letters" /> },
  { path: "documents/analyses", element: <DocumentsPage filter="analyses" /> },
  { path: "documents/generate-resume", element: <ResumeGeneratorPage /> },
  { path: "documents/write-cover-letter", element: <LetterWriterPage /> },
  { path: "documents/analyze", element: <ResumeAnalysisPage /> },
  { path: "ai", element: <AiPage /> },
  { path: "profile", element: <ProfilePage /> },
  ...Object.entries(LEGACY_REDIRECTS).map(([from, to]) => ({
    path: from.slice(1),
    element: <LegacyRedirect to={to} />,
  })),
  { path: "settings/:section", element: <LegacySettings /> },
];

const router = createBrowserRouter([
  {
    path: "/",
    element: <AppShell />,
    children: [...ROUTES, ...designRoutes, { path: "*", element: <Navigate to="/" replace /> }],
  },
]);

export function AppRouter() {
  return <RouterProvider router={router} />;
}
