import { Outlet, useLocation, useNavigate } from "react-router-dom";
import { Suspense, useEffect } from "react";
import { NavRail } from "./NavRail";
import { TopBar } from "./TopBar";
import { SubNav } from "./SubNav";
import { ContextBarProvider } from "./ContextBar";
import { Sections, sectionForPath } from "@/app/router/routes";
import { CommandPalette, useCommandPalette } from "@/shared/ui/CommandPalette";

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
  const navigate = useNavigate();
  const section = sectionForPath(pathname);
  const palette = useCommandPalette();

  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (!(event.metaKey || event.ctrlKey) || event.shiftKey || event.altKey) return;
      const index = Number(event.key);
      if (index < 1 || index > Sections.length) return;
      const target = Sections[index - 1];
      if (!target) return;
      event.preventDefault();
      void navigate(target.routes[0]!.path);
    };
    document.addEventListener("keydown", onKey);
    return () => document.removeEventListener("keydown", onKey);
  }, [navigate]);

  return (
    <div className="flex h-screen min-h-0 overflow-hidden bg-page text-ink">
      <a
        href="#contenu"
        className="sr-only focus:not-sr-only focus:absolute focus:z-50 focus:m-3 focus:rounded-button focus:bg-accent focus:px-3 focus:py-1.5 focus:text-body focus:font-semibold focus:text-on-accent"
      >
        Aller au contenu
      </a>
      <NavRail />
      <ContextBarProvider>
        {(slotRef) => (
          <div className="relative z-0 flex min-w-0 flex-1 flex-col overflow-hidden">
            <TopBar slotRef={slotRef} onOpenPalette={() => palette.setOpen(true)} />
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
      {palette.open ? <CommandPalette onClose={palette.close} /> : null}
    </div>
  );
}
