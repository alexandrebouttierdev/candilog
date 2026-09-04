import { Outlet, useLocation } from "react-router-dom";
import { Suspense } from "react";
import { NavRail } from "./NavRail";
import { TopBar } from "./TopBar";
import { SubNav } from "./SubNav";
import { ContextBarProvider } from "./ContextBar";
import { sectionForPath } from "@/app/router/routes";
import { AiNavigationGuard } from "./AiNavigationGuard";

function PageFallback() {
  return (
    <div className="flex h-full flex-col" role="status" aria-label="Chargement de l'écran">
      <div className="h-topbar flex-none border-b border-line-soft" />
      <div className="min-h-0 flex-1 animate-pulse bg-fill" />
    </div>
  );
}

/** Coque : rail, topbar, sous-navigation, workspace. */
export function AppShell() {
  const { pathname } = useLocation();
  const section = sectionForPath(pathname);

  return (
    <div className="flex h-screen min-h-0 overflow-hidden bg-page text-ink">
      <a
        href="#contenu"
        className="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:m-3 focus:rounded-button focus:bg-accent focus:px-3 focus:py-1.5 focus:text-body focus:font-semibold focus:text-on-accent"
      >
        Aller au contenu
      </a>
      <AiNavigationGuard />
      <NavRail />
      <ContextBarProvider>
        {(slotRef) => (
          <div className="relative z-0 flex min-w-0 flex-1 flex-col overflow-hidden">
            <TopBar slotRef={slotRef} />
            <div className="flex min-h-0 flex-1 overflow-hidden">
              <SubNav section={section} />
              <main
                id="contenu"
                tabIndex={-1}
                className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden"
              >
                <Suspense fallback={<PageFallback />}>
                  <Outlet />
                </Suspense>
              </main>
            </div>
          </div>
        )}
      </ContextBarProvider>
    </div>
  );
}
