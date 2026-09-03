import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, Icon, PageHeader, StatusPill } from "@/shared/ui";
import type { UpdateInfo } from "@/shared/types/generated/settings";
import { useUpdatesViewModel } from "../../viewmodel/useUpdatesViewModel";
import { SettingsBody } from "../components/SettingsUi";

/**
 * Mises à jour : version installée, disponibilité, action.
 *
 * L'écran répond à trois questions et rien de plus — quelle version j'utilise, en existe-t-il
 * une plus récente, que dois-je faire. Le détail du mécanisme de téléchargement, qui n'aide
 * en rien à décider, a été retiré.
 */
export function UpdatesPage() {
  const vm = useUpdatesViewModel();
  const enCours = vm.busy !== null;

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="system_update"
        title="Mises à jour"
        subtitle="Candilog ne vérifie rien sans votre demande"
      />
      <SettingsBody>
        <section className="flex flex-wrap items-start gap-x-10 gap-y-5 border-b border-line-soft pb-6">
          <div className="min-w-[200px] flex-1">
            <p className="text-eyebrow uppercase text-ink-label">Version installée</p>
            <p className="mt-1.5 font-mono tabular text-heading tracking-tight text-ink">
              {vm.version}
            </p>
          </div>

          {vm.update ? (
            <div className="min-w-[200px] flex-1">
              <p className="text-eyebrow uppercase text-ink-label">Nouvelle version</p>
              <p className="mt-1.5 font-mono tabular text-heading tracking-tight text-accent-text">
                {vm.update.version}
              </p>
            </div>
          ) : null}

          <div className="flex flex-none flex-col items-end gap-2.5 pt-0.5">
            <Etat update={vm.update} busy={enCours} />
            {vm.update ? (
              <Button
                variant="primary"
                icon="download"
                disabled={enCours}
                onClick={() => void vm.download()}
              >
                Mettre à jour
              </Button>
            ) : (
              <Button icon="refresh" disabled={enCours} onClick={() => void vm.check()}>
                Rechercher une mise à jour
              </Button>
            )}
          </div>
        </section>

        <Message update={vm.update} busy={vm.busy} error={vm.error} />

        {vm.progress !== null ? (
          <section aria-label="Téléchargement">
            <div
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={vm.progress}
              aria-label="Téléchargement de la mise à jour"
              className="h-1.5 overflow-hidden rounded-pill bg-neutral-tint"
            >
              <div
                className="h-full bg-accent transition-[width] duration-hover"
                style={{ width: `${vm.progress}%` }}
              />
            </div>
            <p className="mt-2 font-mono tabular text-meta text-ink-faint">
              {vm.progress} % téléchargés
            </p>
          </section>
        ) : null}

        {vm.update?.notes ? (
          <section className="min-w-0">
            <p className="text-eyebrow uppercase text-ink-label">Nouveautés</p>
            <p className="mt-2 max-w-2xl text-body leading-relaxed whitespace-pre-line text-ink-muted">
              {vm.update.notes}
            </p>
          </section>
        ) : null}
      </SettingsBody>
    </div>
  );
}

/** Pastille d'état : vérification, à jour, mise à jour disponible, ou rien encore. */
function Etat({ update, busy }: { update: UpdateInfo | null | undefined; busy: boolean }) {
  if (busy) return <StatusPill tone="neutral">Vérification…</StatusPill>;
  if (update) {
    return (
      <StatusPill tone="success" icon="new_releases">
        Mise à jour disponible
      </StatusPill>
    );
  }
  if (update === null) return <StatusPill tone="success">À jour</StatusPill>;
  return null;
}

/** Phrase qui explique l'état courant, ou l'échec de la dernière tentative. */
function Message({
  update,
  busy,
  error,
}: {
  update: UpdateInfo | null | undefined;
  busy: "check" | "download" | null;
  error: string | null;
}) {
  if (error !== null) {
    return (
      <p role="status" className="flex items-start gap-2 text-note leading-relaxed text-danger">
        <Icon name="warning" size={16} className="mt-px flex-none" />
        {error}
      </p>
    );
  }
  if (busy === "download") {
    return (
      <p className="text-note leading-relaxed text-ink-muted">
        Téléchargement en cours. L’installeur sera enregistré dans vos téléchargements ; son
        installation reste à votre main.
      </p>
    );
  }
  if (update) {
    return (
      <div>
        <p className="text-item font-semibold text-ink">Une nouvelle version est disponible</p>
        <p className="mt-1 text-note leading-relaxed text-ink-muted">
          Le téléchargement reste à votre initiative, et l’installation à votre main.
        </p>
      </div>
    );
  }
  if (update === null) {
    return (
      <div>
        <p className="text-item font-semibold text-ink">Candilog est à jour</p>
        <p className="mt-1 text-note leading-relaxed text-ink-muted">
          Vous utilisez la dernière version disponible.
        </p>
      </div>
    );
  }
  return (
    <p className="text-note leading-relaxed text-ink-muted">
      Aucune vérification n’a encore été faite pendant cette session.
    </p>
  );
}
