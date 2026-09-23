import { Suspense, useRef } from "react";
import { useNavigate } from "react-router-dom";
import { cn } from "@/shared/lib/cn";
import { useUiStore } from "@/shared/lib/ui-store";
import type { SettingsSection as Section } from "@/shared/lib/ui-store";
import { useDismissable } from "@/shared/hooks/useDismissable";
import { useFocusTrap } from "@/shared/hooks/useFocusTrap";
import { PATHS } from "@/shared/lib/paths";
import { Button, IconButton, Kbd } from "@/shared/ui";
import {
  AboutPage,
  AppearanceSettings,
  BackupsPage,
  SettingsSection,
  UpdatesPage,
  useAboutViewModel,
} from "@/features/settings";
import { SETTINGS_LABELS, SETTINGS_ORDER } from "./settingsSections";
import { SHORTCUTS } from "./shortcutList";

/**
 * Surcouche Réglages (`DECISIONS.md` B7) : plein cadre sous la barre de titre, ouverte par
 * `⌘,` ou le pied de navigation. Ce n'est pas une destination : la fermer (`Échap`, `✕`)
 * rend l'écran précédent intact.
 *
 * Données, Mises à jour et À propos reprennent encore leurs écrans v1 le temps de leur
 * refonte (étape Réglages du plan) ; leurs fonctions restent donc toutes disponibles.
 */
export function SettingsOverlay() {
  const section = useUiStore((state) => state.settings);
  const open = useUiStore((state) => state.openSettings);
  const close = useUiStore((state) => state.closeSettings);
  const panel = useRef<HTMLDivElement>(null);
  const about = useAboutViewModel();
  useDismissable({ open: section !== null, onDismiss: close });
  useFocusTrap(panel, section !== null);

  if (!section) return null;

  return (
    <div
      ref={panel}
      role="dialog"
      aria-modal="true"
      aria-label="Réglages"
      className="absolute inset-0 z-50 flex animate-pop flex-col bg-app"
    >
      <header className="flex h-overlay-head flex-none items-center gap-2.5 border-b border-bd-soft px-3.5">
        <IconButton icon="close" label="Fermer les réglages" onClick={close} size={13} />
        <h1 className="text-ui font-medium text-tx">Réglages</h1>
        {about.isLoading || about.error ? null : (
          <span className="ml-auto font-mono text-caps text-tx-6">Candilog {about.version}</span>
        )}
      </header>

      <div className="flex min-h-0 flex-1 justify-start min-[1440px]:justify-center">
        <nav
          aria-label="Sections des réglages"
          className="w-[206px] flex-none border-r border-bd-soft p-2 min-[1440px]:border-l"
        >
          {SETTINGS_ORDER.map((key) => (
            <button
              key={key}
              type="button"
              aria-current={key === section ? "page" : undefined}
              onClick={() => open(key)}
              className={cn(
                "flex h-[29px] w-full items-center rounded-r6 px-2.5 text-left text-ui transition-color",
                key === section
                  ? "bg-sel font-medium text-tx shadow-[inset_2px_0_0_var(--ac)]"
                  : "text-tx-3 hover:bg-elev hover:text-tx",
              )}
            >
              {SETTINGS_LABELS[key]}
            </button>
          ))}
        </nav>
        <div className="min-w-0 flex-1 overflow-y-auto px-[26px] py-5 min-[1440px]:max-w-[652px]">
          <div className="max-w-[600px]">
            <Suspense fallback={null}>
              <SectionContent section={section} />
            </Suspense>
          </div>
        </div>
      </div>

      <footer className="flex h-statusbar flex-none items-center border-t border-bd-soft px-3.5">
        <p className="font-mono text-caps text-tx-5">appliqué immédiatement · propre à cet ordinateur</p>
        <p className="ml-auto flex items-center gap-1.5 text-sub text-tx-3">
          Fermer <Kbd shortcut="escape" />
        </p>
      </footer>
    </div>
  );
}

function SectionContent({ section }: { section: Section }) {
  const navigate = useNavigate();
  const close = useUiStore((state) => state.closeSettings);

  switch (section) {
    case "appearance":
      return <AppearanceSettings />;
    case "data":
      return <BackupsPage />;
    case "ai":
      return (
        <SettingsSection
          title="Intelligence artificielle"
          description="Fournisseurs, modèles et test de l'IA se règlent dans l'écran Intelligence artificielle."
        >
          <Button
            variant="secondary"
            onClick={() => {
              close();
              void navigate(PATHS.ai);
            }}
          >
            Ouvrir Intelligence artificielle
          </Button>
        </SettingsSection>
      );
    case "shortcuts":
      return (
        <SettingsSection
          title="Raccourcis"
          description="Le contrat clavier de l'écran courant est toujours rappelé en bas à droite de la fenêtre."
        >
          <table className="w-full text-ui">
            <tbody>
              {SHORTCUTS.map((item) => (
                <tr key={item.effect} className="border-b border-bd-soft">
                  <td className="py-2.5 text-tx-2">{item.effect}</td>
                  <td className="py-2.5 text-sub text-tx-5">{item.context}</td>
                  <td className="py-2.5 text-right">
                    <span className="inline-flex gap-1">
                      {item.keys.map((key) => (
                        <Kbd key={key} shortcut={key} />
                      ))}
                    </span>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </SettingsSection>
      );
    case "updates":
      return <UpdatesPage />;
    case "about":
      return <AboutPage />;
  }
}
