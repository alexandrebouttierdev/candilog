import type { Entreprise } from "../../services/entreprise.service";
import { Button, Icon } from "@/shared/ui";

/**
 * Fiche détaillée d'une entreprise.
 *
 * Structure imposée par le guide : bandeau d'identité, colonne principale, colonne latérale
 * d'informations. Les champs non renseignés sont **explicitement marqués** plutôt que
 * masqués — le guide le demande, et une fiche dont les lignes disparaissent au gré du
 * remplissage ne se lit pas d'un coup d'œil.
 */
export function EntrepriseDetail({
  entreprise,
  onEdit,
  onDelete,
}: {
  entreprise: Entreprise;
  onEdit: () => void;
  onDelete: () => void;
}) {
  return (
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex flex-none items-start gap-3 border-b border-line bg-surface-alt px-6 py-5">
        <span className="flex size-11 flex-none items-center justify-center rounded-card bg-accent-tint text-accent">
          <Icon name="apartment" size={22} />
        </span>
        <div className="min-w-0 flex-1">
          <h2 className="truncate text-title">{entreprise.nom}</h2>
          <p className="truncate text-meta text-ink-muted">
            {[entreprise.secteur, entreprise.type, entreprise.ville]
              .filter(Boolean)
              .join(" · ") || "Aucune information complémentaire"}
          </p>
        </div>
        <Button icon="edit" onClick={onEdit}>
          Modifier
        </Button>
        <Button variant="danger" icon="delete" onClick={onDelete}>
          Supprimer
        </Button>
      </header>

      <div className="grid flex-1 grid-cols-[1fr_280px] gap-6 p-6">
        <section className="flex flex-col gap-3">
          <h3 className="text-eyebrow uppercase text-ink-faint">Notes</h3>
          <div className="rounded-card border border-line bg-surface p-4">
            {entreprise.notes ? (
              <p className="text-body whitespace-pre-wrap text-ink">{entreprise.notes}</p>
            ) : (
              <p className="text-meta text-ink-faint">
                Aucune note. Utilisez « Modifier » pour consigner le contexte, la culture ou
                les informations utiles.
              </p>
            )}
          </div>
        </section>

        <aside className="flex flex-col gap-3">
          <h3 className="text-eyebrow uppercase text-ink-faint">Informations</h3>
          <dl className="flex flex-col gap-3 rounded-card border border-line bg-surface p-4">
            <Ligne label="Secteur" valeur={entreprise.secteur} />
            <Ligne label="Type" valeur={entreprise.type} />
            <Ligne label="Ville" valeur={entreprise.ville} />
            <Ligne label="Adresse" valeur={entreprise.adresse} />
            <Ligne label="Site web" valeur={entreprise.siteWeb} lien />
          </dl>
        </aside>
      </div>
    </div>
  );
}

function Ligne({
  label,
  valeur,
  lien = false,
}: {
  label: string;
  valeur: string | null;
  lien?: boolean;
}) {
  return (
    <div className="flex flex-col gap-0.5">
      <dt className="text-label text-ink-faint">{label}</dt>
      <dd className="text-body break-words text-ink">
        {valeur ? (
          lien ? (
            // `rel` et `target` explicites : l'application est servie depuis un contexte
            // local, un lien externe sans `noreferrer` exposerait son origine.
            <a
              href={valeur}
              target="_blank"
              rel="noreferrer noopener"
              className="text-accent underline-offset-2 hover:underline"
            >
              {valeur}
            </a>
          ) : (
            valeur
          )
        ) : (
          <span className="text-ink-faint">Non renseigné</span>
        )}
      </dd>
    </div>
  );
}
