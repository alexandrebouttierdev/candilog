import { Outlet } from "react-router-dom";
import { Suspense } from "react";
import { NavRail } from "./NavRail";
import { ContextTabs } from "./ContextTabs";
import { ContextBarProvider } from "./ContextBar";

function PageFallback() {
  return (
    <div className="flex h-full flex-col" role="status" aria-label="Chargement de l'écran">
      <div className="h-[69px] flex-none border-b border-line bg-surface" />
      <div className="min-h-0 flex-1 animate-pulse bg-neutral-tint/40" />
    </div>
  );
}

/**
 * Coque applicative : rail, onglets contextuels, contenu.
 *
 * Une seule zone défilante, celle de l'écran (guide SPECDESIGN, section 7) : le rail et les
 * onglets sont fixes. `main` ne défile pas lui-même, pour éviter un scroll imbriqué avec
 * les pages qui gèrent déjà leur en-tête collant.
 */
export function AppShell() {
  return (
    <div className="flex h-screen min-h-0 overflow-hidden bg-page text-ink">
      <a
        href="#contenu"
        className="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:m-3 focus:rounded-button focus:bg-accent focus:px-3.5 focus:py-2 focus:text-body focus:font-medium focus:text-white"
      >
        Aller au contenu
      </a>
      <NavRail />
      <ContextBarProvider>
        {(slotRef) => (
          <div className="flex min-w-0 flex-1 flex-col overflow-hidden">
            <ContextTabs slotRef={slotRef} />
            <main id="contenu" tabIndex={-1} className="flex min-h-0 flex-1 flex-col overflow-hidden">
              <Suspense fallback={<PageFallback />}>
                <Outlet />
              </Suspense>
            </main>
          </div>
        )}
      </ContextBarProvider>
    </div>
  );
}
