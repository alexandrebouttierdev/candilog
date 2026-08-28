import type { Contact } from "../../services/contact.service";
import { roleMeta } from "../../model/roles";
import { Card, CardHeader, RecordAction, RecordHeader, StatusPill, initiales } from "@/shared/ui";

/**
 * Fiche détaillée d'un contact du réseau.
 *
 * Même disposition que la fiche entreprise — bandeau d'identité, colonne principale, colonne
 * d'informations — la pastille en rond parce qu'il s'agit d'une personne, comme les
 * maquettes Relations le distinguent.
 */
export function ContactDetail({
  contact,
  onEdit,
  onDelete,
}: {
  contact: Contact;
  onEdit: () => void;
  onDelete: () => void;
}) {
  const nomComplet = `${contact.prenom} ${contact.nom}`;

  return (
    <div className="min-w-0 flex-1 overflow-y-auto bg-page">
      <RecordHeader
        round
        initials={initiales(contact.prenom, contact.nom)}
        title={nomComplet}
        badge={
          contact.roleSuivi ? (
            <StatusPill tone="accent" icon={roleMeta(contact.roleSuivi).icon}>
              {contact.roleSuivi}
            </StatusPill>
          ) : null
        }
        subtitle={
          [contact.poste, contact.entrepriseNom].filter(Boolean).join(" · ") ||
          "Aucun contexte professionnel renseigné"
        }
        actions={
          <>
            {contact.email ? (
              <RecordAction
                icon="mail"
                onClick={() => {
                  window.location.href = `mailto:${contact.email}`;
                }}
              >
                E-mail
              </RecordAction>
            ) : null}
            {contact.linkedin ? (
              <RecordAction
                icon="link"
                onClick={() => window.open(contact.linkedin ?? "", "_blank", "noopener")}
              >
                LinkedIn
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
      />

      <div className="flex flex-wrap items-start gap-4 px-[26px] pt-5 pb-[30px]">
        <Card clipped className="min-w-0 flex-[1_1_420px]">
          <CardHeader compact>Notes</CardHeader>
          <div className="px-[17px] py-3.5">
            {contact.notes ? (
              <p className="text-body leading-normal whitespace-pre-wrap text-ink">
                {contact.notes}
              </p>
            ) : (
              <p className="text-label leading-normal text-ink-faint">
                Aucune note. Utilisez « Modifier » pour consigner les sujets abordés et les
                points à retenir.
              </p>
            )}
          </div>
        </Card>

        <Card clipped className="max-w-[360px] flex-[1_1_280px]">
          <CardHeader compact>Informations</CardHeader>
          <div className="px-[17px] pt-1 pb-3">
            <Ligne label="Entreprise" valeur={contact.entrepriseNom} />
            <Ligne label="Poste" valeur={contact.poste} />
            <Ligne
              label="E-mail"
              valeur={contact.email}
              href={contact.email ? `mailto:${contact.email}` : null}
            />
            <Ligne
              label="Téléphone"
              valeur={contact.telephone}
              href={contact.telephone ? `tel:${contact.telephone}` : null}
            />
            <Ligne label="LinkedIn" valeur={contact.linkedin} href={contact.linkedin} />
            <Ligne label="Rôle" valeur={contact.roleSuivi} />
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
  href = null,
}: {
  label: string;
  valeur: string | null;
  href?: string | null;
}) {
  return (
    <div className="flex items-center justify-between gap-3.5 border-b border-line py-[9px] last:border-b-0">
      <span className="flex-none text-note text-ink-faint">{label}</span>
      <span className="min-w-0 flex-1 truncate text-right text-body font-medium text-ink">
        {valeur ? (
          href ? (
            <a
              href={href}
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
