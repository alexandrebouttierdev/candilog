import { lazy } from "react";
import { createBrowserRouter, RouterProvider, Navigate } from "react-router-dom";
import type { RouteObject } from "react-router-dom";
import { AppShell } from "@/app/layout/AppShell";
import { Sections } from "./routes";
import { PlaceholderPage } from "./PlaceholderPage";

const DashboardPage = lazy(() =>
  import("@/features/analytics/view/pages/DashboardPage").then((m) => ({ default: m.DashboardPage })),
);
const AnalyticsPage = lazy(() =>
  import("@/features/analytics/view/pages/AnalyticsPage").then((m) => ({ default: m.AnalyticsPage })),
);
const ApplicationsPage = lazy(() =>
  import("@/features/applications/view/pages/ApplicationsPage").then((m) => ({
    default: m.ApplicationsPage,
  })),
);
const CalendarPage = lazy(() =>
  import("@/features/calendar/view/pages/CalendarPage").then((m) => ({
    default: m.CalendarPage,
  })),
);
const CompaniesPage = lazy(() =>
  import("@/features/companies/view/pages/CompaniesPage").then((m) => ({
    default: m.CompaniesPage,
  })),
);
const NetworkPage = lazy(() =>
  import("@/features/contacts/view/pages/NetworkPage").then((m) => ({ default: m.NetworkPage })),
);
const ProfilePage = lazy(() =>
  import("@/features/profile/view/pages/ProfilePage").then((m) => ({ default: m.ProfilePage })),
);
const ResumeLibraryPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({ default: m.ResumeLibraryPage })),
);
const ResumeGeneratorPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({
    default: m.ResumeGeneratorPage,
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
const ResumeAnalysisPage = lazy(() =>
  import("@/features/documents/view/pages/DocumentsPages").then((m) => ({
    default: m.ResumeAnalysisPage,
  })),
);
const AiPage = lazy(() =>
  import("@/features/settings/view/pages/AiPage").then((m) => ({ default: m.AiPage })),
);
const BackupsPage = lazy(() =>
  import("@/features/settings/view/pages/BackupsPage").then((m) => ({
    default: m.BackupsPage,
  })),
);
const UpdatesPage = lazy(() =>
  import("@/features/settings/view/pages/UpdatesPage").then((m) => ({
    default: m.UpdatesPage,
  })),
);
const AboutPage = lazy(() =>
  import("@/features/settings/view/pages/AboutPage").then((m) => ({ default: m.AboutPage })),
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
const Pages: Record<string, React.ReactElement> = {
  "/": <DashboardPage />,
  "/analytics": <AnalyticsPage />,
  "/tracking/applications": <ApplicationsPage />,
  "/tracking/calendar": <CalendarPage />,
  "/relations/companies": <CompaniesPage />,
  "/relations/network": <NetworkPage />,
  "/profile": <ProfilePage />,
  "/documents/cv": <ResumeLibraryPage />,
  "/documents/generate-resume": <ResumeGeneratorPage />,
  "/documents/cover-letters": <LettersLibraryPage />,
  "/documents/write-cover-letter": <LetterWriterPage />,
  "/documents/analyze": <ResumeAnalysisPage />,
  "/settings/ai": <AiPage />,
  "/settings/backups": <BackupsPage />,
  "/settings/updates": <UpdatesPage />,
  "/settings/about": <AboutPage />,
};

/** Paths dont la page n'est plus un jalon. */
export const MIGRATED_PATHS = Object.keys(Pages);

const screenRoutes: RouteObject[] = Sections.flatMap((section) =>
  section.routes.map((route): RouteObject => {
    const element = Pages[route.path] ?? (
      <PlaceholderPage icon={route.icon} title={route.label} section={section.long_label} />
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
