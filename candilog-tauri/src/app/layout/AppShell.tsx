import { Outlet } from "react-router-dom";
import { NavRail } from "./NavRail";
import { ContextTabs } from "./ContextTabs";

/**
 * Coque applicative : rail, onglets contextuels, contenu.
 *
 * Une seule zone défilante, celle du contenu (guide SPECDESIGN, section 7) : le rail et les
 * onglets sont fixes, ce qui évite le défilement imbriqué qu'imposerait un `overflow` sur
 * la coque entière.
 */
export function AppShell() {
  return (
    <div className="flex h-screen overflow-hidden bg-page text-ink">
      <NavRail />
      <div className="flex min-w-0 flex-1 flex-col">
        <ContextTabs />
        <main className="min-h-0 flex-1 overflow-y-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
