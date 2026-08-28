import type { Entreprise } from "../../services/entreprise.service";
import type { Candidature } from "@/features/candidatures/services/candidature.service";
import { contratLabel, statutMeta } from "@/features/candidatures/model/statuts";
import { versDateLongue } from "@/shared/lib/dates";
import {
  Card,
  CardHeader,
  CardLink,
  EmptyState,
  Icon,
  RecordAction,
  RecordHeader,
  RecordStat,
  StatusPill,
  initialesMot,
} from "@/shared/ui";

/**
 * Fiche détaillée d'une entreprise.
 *
 * Disposition des maquettes Relations : bandeau d'identité et chiffres clés, colonne
 * principale (candidatures liées, notes) et colonne latérale d'informations.
 *
 * Les champs non renseignés sont **explicitement marqués** plutôt que masqués — le guide le
 * demande, et une fiche dont les lignes disparaissent au gré du remplissage ne se lit pas
 * d'un coup d'œil.
 */
export function EntrepriseDetail({
  entreprise,
  candidatures,
  totalCandidatures,
  onEdit,
  onDelete,
  onOuvrirCandidature,
  onToutVoir,
}: {
  entreprise: Entreprise;
  candidatures: readonly Candidature[];
  totalCandidatures: number;
  onEdit: () => void;
  onDelete: () => void;
  onOuvrirCandidature: (candidature: Candidature) => void;
  onToutVoir: () => void;
}) {
  const entretiens = candidatures.filter((item) => item.statut === "ENTRETIEN").length;
  const enAttente = candidatures.filter((item) => item.statut === "EN_ATTENTE").length;

  return (
    <div className="min-w-0 flex-1 overflow-y-auto bg-page">
      <RecordHeader
        initials={initialesMot(entreprise.nom)}
        title={entreprise.nom}
        badge={
          entreprise.type ? (
            <StatusPill tone="accent" icon="business_center">
              {entreprise.type}
            </StatusPill>
          ) : null
        }
        subtitle={
          [entreprise.secteur, entreprise.ville].filter(Boolean).join(" · ") ||
          "Aucune information complémentaire"
        }
        actions={
          <>
            {entreprise.siteWeb ? (
              <RecordAction
                icon="open_in_new"
                onClick={() => window.open(entreprise.siteWeb ?? "", "_blank", "noopener")}
              >
                Site web
              </RecordAction>
            ) : null}
            <RecordAction icon="edit" onClick={onEdit}>
              Modifier
            </RecordAction>
            <RecordAction icon="delete" onClick={onDelete}>
              Supprimer
            </RecordAction>
          </>
        }
        stats={
          <>
            <RecordStat icon="work" iconClassName="text-accent" label="Candidatures">
              {totalCandidatures}
            </RecordStat>
            <RecordStat icon="event_available" iconClassName="text-success" label="Entretiens">
              {entretiens}
            </RecordStat>
            <RecordStat icon="hourglass_top" label="En attente">
              {enAttente}
            </RecordStat>
          </>
        }
      />

      <div className="flex flex-wrap items-start gap-4 px-[26px] pt-5 pb-[30px]">
        <div className="flex min-w-0 flex-[1_1_420px] flex-col gap-4">
          <Card clipped>
            <CardHeader
              compact
              meta={
                candidatures.length > 0 ? (
                  <CardLink compact onClick={onToutVoir}>
                    Tout voir
                  </CardLink>
                ) : undefined
              }
            >
              Candidatures liées
            </CardHeader>
            {candidatures.length === 0 ? (
              <EmptyState
                icon="work_off"
                title="Aucune candidature"
                description="Les candidatures envoyées à cette société apparaîtront ici."
              />
            ) : (
              candidatures.map((candidature) => {
                const statut = statutMeta(candidature.statut);
                return (
                  <button
                    key={candidature.id}
                    type="button"
                    onClick={() => onOuvrirCandidature(candidature)}
                    className="flex w-full items-center gap-[11px] border-b border-line px-[17px] py-[11px] text-left transition-colors duration-150 last:border-b-0 hover:bg-neutral-tint"
                  >
                    <span className="flex size-7 flex-none items-center justify-center rounded-button bg-neutral-tint text-ink-muted">
                      <Icon name="work" size={16} />
                    </span>
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-body font-mid text-ink">
                        {candidature.poste}
                      </span>
                      <span className="mt-px block truncate text-meta text-ink-faint">
                        {contratLabel(candidature.typeContrat)} · envoyée le{" "}
                        {versDateLongue(candidature.dateEnvoi)}
                      </span>
                    </span>
                    <StatusPill tone={statut.tone} compact>
                      {statut.label}
                    </StatusPill>
                  </button>
                );
              })
            )}
          </Card>

          <Card clipped>
            <CardHeader compact>Notes</CardHeader>
            <div className="px-[17px] py-3.5">
              {entreprise.notes ? (
                <p className="text-body leading-normal whitespace-pre-wrap text-ink">
                  {entreprise.notes}
                </p>
              ) : (
                <p className="text-label leading-normal text-ink-faint">
                  Aucune note. Utilisez « Modifier » pour consigner le contexte, la culture ou
                  les informations utiles.
                </p>
              )}
            </div>
          </Card>
        </div>

        <Card clipped className="max-w-[360px] flex-[1_1_280px]">
          <CardHeader compact>Informations</CardHeader>
          <div className="px-[17px] pt-1 pb-3">
            <Ligne label="Secteur" valeur={entreprise.secteur} />
            <Ligne label="Type" valeur={entreprise.type} />
            <Ligne label="Ville" valeur={entreprise.ville} />
            <Ligne label="Adresse" valeur={entreprise.adresse} />
            <Ligne label="Site web" valeur={entreprise.siteWeb} lien />
            <Ligne label="Ajoutée le" valeur={versDateLongue(entreprise.createdAt, true)} />
          </div>
        </Card>
      </div>
    </div>
  );
}

/** Rangée libellé / valeur de la carte « Informations » : 9 px de padding, filet bas. */
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
    <div className="flex items-center justify-between gap-3.5 border-b border-line py-[9px] last:border-b-0">
      <span className="flex-none text-note text-ink-faint">{label}</span>
      <span className="min-w-0 flex-1 truncate text-right text-body font-medium text-ink">
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
          <span className="font-normal text-ink-faint">Non renseigné</span>
        )}
      </span>
    </div>
  );
}
