import type { Contact } from "../../services/contact.service";
import { Button, Icon, StatusPill, initiales } from "@/shared/ui";

/** Fiche détaillée d'un contact du réseau. */
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
    <div className="flex h-full flex-col overflow-y-auto">
      <header className="flex flex-none items-start gap-3 border-b border-line bg-surface-alt px-6 py-5">
        <span
          aria-hidden="true"
          className="flex size-11 flex-none items-center justify-center rounded-card bg-accent-tint text-section text-accent"
        >
          {initiales(contact.prenom, contact.nom)}
        </span>
        <div className="min-w-0 flex-1">
          <div className="flex items-center gap-2">
            <h2 className="truncate text-title">{nomComplet}</h2>
            {contact.roleSuivi ? (
              <StatusPill tone="accent" icon="badge">
                {contact.roleSuivi}
              </StatusPill>
            ) : null}
          </div>
          <p className="truncate text-meta text-ink-muted">
            {[contact.poste, contact.entrepriseNom].filter(Boolean).join(" · ") ||
              "Aucun contexte professionnel renseigné"}
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
            {contact.notes ? (
              <p className="text-body whitespace-pre-wrap text-ink">{contact.notes}</p>
            ) : (
              <p className="text-meta text-ink-faint">
                Aucune note. Utilisez « Modifier » pour consigner les sujets abordés et les
                points à retenir.
              </p>
            )}
          </div>
        </section>

        <aside className="flex flex-col gap-3">
          <h3 className="text-eyebrow uppercase text-ink-faint">Coordonnées</h3>
          <dl className="flex flex-col gap-3 rounded-card border border-line bg-surface p-4">
            <Ligne icone="mail" label="E-mail" valeur={contact.email} href={contact.email ? `mailto:${contact.email}` : null} />
            <Ligne icone="call" label="Téléphone" valeur={contact.telephone} href={contact.telephone ? `tel:${contact.telephone}` : null} />
            <Ligne icone="link" label="LinkedIn" valeur={contact.linkedin} href={contact.linkedin} />
            <Ligne icone="apartment" label="Entreprise" valeur={contact.entrepriseNom} />
            <Ligne icone="work" label="Poste" valeur={contact.poste} />
            <Ligne icone="badge" label="Rôle dans le suivi" valeur={contact.roleSuivi} />
          </dl>
        </aside>
      </div>
    </div>
  );
}

function Ligne({
  icone,
  label,
  valeur,
  href = null,
}: {
  icone: string;
  label: string;
  valeur: string | null;
  href?: string | null;
}) {
  return (
    <div className="flex gap-2">
      <Icon name={icone} size={15} className="mt-0.5 flex-none text-ink-faint" />
      <div className="min-w-0 flex-1">
        <dt className="text-label text-ink-faint">{label}</dt>
        <dd className="text-body break-words text-ink">
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
            <span className="text-ink-faint">Non renseigné</span>
          )}
        </dd>
      </div>
    </div>
  );
}
