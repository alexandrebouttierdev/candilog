import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, PageHeader, StatusPill } from "@/shared/ui";
import { useUpdatesViewModel } from "../../viewmodel/useUpdatesViewModel";
import { ActionCard, SettingsBody, SettingsCard, SettingsHero } from "../components/SettingsUi";

/** Recherche et installation assistée des mises à jour. */
export function UpdatesPage() {
  const vm = useUpdatesViewModel();

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Candilog · données locales</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="system_update"
        title="Mises à jour"
        subtitle={`Version actuelle ${vm.version}`}
      />
      <SettingsBody>
        <SettingsHero
          kicker="Version actuelle"
          title={vm.version}
          description="Candilog vérifie les nouvelles versions uniquement lorsque vous le demandez."
        />
        <div className="grid gap-4 md:grid-cols-2">
          <ActionCard
            icon="refresh"
            title="Disponibilité"
            description="Interrogez la source officielle et comparez-la à votre version installée."
          >
            <div className="space-y-3">
              {vm.update ? (
                <StatusPill tone="success" icon="new_releases">
                  {`Version ${vm.update.version} disponible`}
                </StatusPill>
              ) : vm.update === null ? (
                <StatusPill tone="neutral">Aucune mise à jour en attente</StatusPill>
              ) : null}
              {vm.update ? (
                <Button variant="primary" icon="download" disabled={vm.busy !== null} onClick={() => void vm.download()}>
                  Télécharger la mise à jour
                </Button>
              ) : (
                <Button variant="primary" icon="refresh" disabled={vm.busy !== null} onClick={() => void vm.check()}>
                  Rechercher maintenant
                </Button>
              )}
              {vm.error ? <p className="text-meta text-danger">{vm.error}</p> : null}
            </div>
          </ActionCard>
          <ActionCard
            icon="verified"
            title="Installation maîtrisée"
            description="L’installeur adapté à votre système est téléchargé, son empreinte SHA-256 comparée à celle publiée avec la release, et il n’est enregistré qu’ensuite."
          >
            <p className="text-meta text-ink-muted">Aucune installation silencieuse</p>
            <p className="mt-1 text-meta text-ink-faint">
              L’empreinte atteste que le fichier est arrivé intact, et l’attestation de
              provenance publiée avec la release qu’il vient bien du dépôt officiel. Ni
              l’une ni l’autre n’est une signature de code : votre système peut afficher un
              avertissement « éditeur inconnu » à l’installation.
            </p>
          </ActionCard>
        </div>
        {vm.progress !== null ? (
          <SettingsCard icon="download" title="Téléchargement">
            <div
              role="progressbar"
              aria-valuemin={0}
              aria-valuemax={100}
              aria-valuenow={vm.progress}
              aria-label="Téléchargement de la mise à jour"
              className="h-2 overflow-hidden rounded-pill bg-neutral-tint"
            >
              <div className="h-full bg-accent" style={{ width: `${vm.progress}%` }} />
            </div>
            <p className="mt-2 text-meta text-ink-muted">{vm.progress} %</p>
          </SettingsCard>
        ) : null}
      </SettingsBody>
    </div>
  );
}
