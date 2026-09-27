import { useState } from "react";
import type { ReactNode } from "react";
import {
  Avatar,
  Button,
  ConfirmDialog,
  EmptyState,
  ErrorBanner,
  Kbd,
  LineIcon,
  SegmentedControl,
  Skeleton,
  StatusGlyph,
  Switch,
} from "@/shared/ui";
import type { DialogRegister } from "@/shared/ui";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ThemePref } from "@/shared/lib/ui-store";

/**
 * Planche de vérification du design system v2.
 *
 * Sert à comparer les primitives à `docs/DESIGN.md` et à `reference_design/` dans les deux
 * thèmes, et à éprouver les états au clavier. N'est atteignable que par l'URL `/_design`,
 * jamais depuis la navigation : c'est un outil de revue, pas un écran de l'application.
 */

const THEMES: ReadonlyArray<{ value: ThemePref; label: string }> = [
  { value: "light", label: "Clair" },
  { value: "dark", label: "Sombre" },
  { value: "system", label: "Système" },
];

const DIALOGS: Record<DialogRegister, { title: string; description: string; confirm: string }> = {
  destruction: {
    title: "Supprimer CAN-142 ?",
    description: "La candidature, sa relance et son historique seront supprimés.",
    confirm: "Supprimer",
  },
  confirmation: {
    title: "Interrompre la génération ?",
    description: "Le document en cours sera perdu.",
    confirm: "Interrompre",
  },
  information: {
    title: "Exporter les relations",
    description: "Entreprises et contacts sont exportés dans deux fichiers distincts.",
    confirm: "Exporter les fichiers",
  },
};

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section aria-label={title} className="rounded-r9 bg-group p-3.5">
      <h2 className="caps mb-3">{title}</h2>
      {children}
    </section>
  );
}

export function DesignGallery() {
  const { theme, setTheme, notify } = useUiStore();
  const [dialog, setDialog] = useState<DialogRegister | null>(null);
  const [switched, setSwitched] = useState(true);
  const current = dialog ? DIALOGS[dialog] : null;

  return (
    <div className="flex h-full flex-col">
      <div className="flex h-toolbar flex-none items-center gap-2 border-b border-bd-soft px-3.5">
        <span className="text-ui font-medium text-tx">Design system</span>
        <span className="text-small text-tx-5">planche de vérification</span>
        <span className="ml-auto">
          <SegmentedControl dense label="Thème" value={theme} onChange={setTheme} options={THEMES} />
        </span>
      </div>

      <div className="flex min-h-0 flex-1 flex-col gap-2.5 overflow-y-auto p-3.5">
        <Section title="Boutons">
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="primary" shortcut="n">
              Ajouter une candidature
            </Button>
            <Button>Secondaire</Button>
            <Button variant="ghost">Discret</Button>
            <Button variant="danger" onClick={() => setDialog("destruction")}>
              Supprimer…
            </Button>
            <Button variant="link">Lien d’action</Button>
            <Button variant="primary" disabled>
              Désactivé
            </Button>
            <Button size="compact">
              <LineIcon name="export-csv" size={13} />
              CSV
            </Button>
          </div>
        </Section>

        <Section title="Statuts, touches, avatars">
          <div className="flex flex-wrap items-center gap-4">
            <span className="flex items-center gap-2 text-small text-tx-2">
              <StatusGlyph tone="n" /> En attente
            </span>
            <span className="flex items-center gap-2 text-small text-tx-2">
              <StatusGlyph tone="a" /> Relancée
            </span>
            <span className="flex items-center gap-2 text-small text-tx-2">
              <StatusGlyph tone="g" /> Entretien
            </span>
            <span className="flex items-center gap-2 text-small text-tx-2">
              <StatusGlyph tone="c" /> Refusée
            </span>
            <Kbd shortcut="mod+k" />
            <Kbd shortcut="g a" />
            <Avatar name="Novéa Services" kind="company" />
            <Avatar name="Claire Ménard" kind="person" />
            <Switch checked={switched} onChange={setSwitched} label="Animations" />
          </div>
        </Section>

        <Section title="Dialogues">
          <div className="flex flex-wrap gap-2">
            {(Object.keys(DIALOGS) as DialogRegister[]).map((register) => (
              <Button key={register} onClick={() => setDialog(register)}>
                Registre {register}
              </Button>
            ))}
            <Button onClick={() => notify({ tone: "success", title: "CAN-142 → Entretien" })}>Notification</Button>
          </div>
        </Section>

        <Section title="États">
          <div className="grid gap-2.5 [grid-template-columns:repeat(auto-fit,minmax(240px,1fr))]">
            <EmptyState title="Aucune candidature" description="Votre suivi commence ici." />
            <div className="flex flex-col gap-2">
              {[0, 1, 2].map((index) => (
                <Skeleton key={index} index={index} className="h-4" />
              ))}
            </div>
            <ErrorBanner message="La base est verrouillée par une autre fenêtre." onRetry={() => undefined} />
          </div>
        </Section>
      </div>

      <ConfirmDialog
        open={current !== null}
        register={dialog ?? "information"}
        title={current?.title ?? ""}
        description={current?.description ?? ""}
        confirmLabel={current?.confirm ?? ""}
        onCancel={() => setDialog(null)}
        onConfirm={() => setDialog(null)}
      />
    </div>
  );
}
