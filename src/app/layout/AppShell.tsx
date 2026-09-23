import { Outlet } from "react-router-dom";
import { Suspense } from "react";
import { ChromeProvider } from "@/shared/lib/chrome";
import { CommandProvider } from "@/shared/lib/commands";
import { AiConfigBridge, AiRequiredModal } from "@/features/ai";
import { SettingsOverlay } from "@/app/overlays/SettingsOverlay";
import { CommandPalette } from "@/app/palette/CommandPalette";
import { useShellCommands } from "@/app/palette/useShellCommands";
import { AiNavigationGuard } from "./AiNavigationGuard";
import { Sidebar } from "./Sidebar";
import { StatusBar } from "./StatusBar";
import { TitleBar } from "./TitleBar";
import { useShellShortcuts } from "./useShellShortcuts";

function PageFallback() {
  return (
    <div className="flex h-full flex-col" role="status" aria-label="Chargement de l'écran">
      <div className="h-toolbar flex-none border-b border-bd-soft" />
      <div className="min-h-0 flex-1 animate-sk bg-sk/40" />
    </div>
  );
}

/**
 * Coque v2 (`reference_design/DESIGN_SYSTEM.md` §9) : barre de titre de 40 px sur toute la
 * largeur, navigation à gauche, panneau de contenu, barre d'état de 34 px. Les barres ne
 * défilent jamais ; seule la zone de contenu défile. La surcouche Réglages recouvre tout
 * sous la barre de titre, qui ne disparaît jamais (`PLATFORM.md` C2).
 */
export function AppShell() {
  return (
    <CommandProvider>
      <ChromeProvider>
        <ShellFrame />
      </ChromeProvider>
    </CommandProvider>
  );
}

function ShellFrame() {
  useShellShortcuts();
  useShellCommands();

  return (
    <div className="flex h-screen min-h-0 flex-col overflow-hidden bg-app text-tx-2">
      <a
        href="#contenu"
        className="sr-only focus:not-sr-only focus:absolute focus:z-[90] focus:m-3 focus:rounded-r7 focus:bg-ac focus:px-3 focus:py-1.5 focus:text-ui focus:font-medium focus:text-white"
      >
        Aller au contenu
      </a>
      <AiNavigationGuard />
      <AiConfigBridge />
      <AiRequiredModal />
      <TitleBar />
      <div className="relative flex min-h-0 flex-1 flex-col">
        <div className="flex min-h-0 flex-1">
          <Sidebar />
          <main
            id="contenu"
            tabIndex={-1}
            className="flex min-h-0 min-w-0 flex-1 flex-col overflow-hidden rounded-tl-r9 border-t border-l border-bd bg-panel outline-none"
          >
            <Suspense fallback={<PageFallback />}>
              <Outlet />
            </Suspense>
          </main>
        </div>
        <StatusBar />
        <SettingsOverlay />
      </div>
      <CommandPalette />
    </div>
  );
}
