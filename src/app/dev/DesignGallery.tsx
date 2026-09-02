import { useState } from "react";
import {
  Button,
  ConfirmDialog,
  DataTable,
  EmptyState,
  ErrorBanner,
  FormField,
  ModalHost,
  PageHeader,
  Pager,
  Select,
  SkeletonRows,
  StatCard,
  StatusPill,
  TextArea,
  TextInput,
  TimelineList,
} from "@/shared/ui";
import type { Column, Tone } from "@/shared/ui";
import { useUiStore } from "@/shared/lib/ui-store";
import type { ThemePref } from "@/shared/lib/ui-store";
import type { IconName } from "@/shared/ui/icon-names";

/**
 * Planche de vérification du design system.
 *
 * Sert à comparer les composants aux règles de `docs/DESIGN.md` dans les deux thèmes et à
 * éprouver les états au clavier. N'est atteignable que par l'URL `/_design`, jamais depuis
 * la navigation : c'est un outil de revue, pas un écran de l'application.
 */

interface DemoRow {
  id: string;
  job_title: string;
  company: string;
  contract: string;
  status: { label: string; tone: Tone; icon: IconName };
  date: string;
}

const ROWS: DemoRow[] = [
  {
    id: "1",
    job_title: "Développeur Frontend",
    company: "Nova Digital",
    contract: "CDI",
    status: { label: "Entretien", tone: "success", icon: "event_available" },
    date: "20 août",
  },
  {
    id: "2",
    job_title: "Product Designer",
    company: "Atlas Studio",
    contract: "CDI",
    status: { label: "En attente", tone: "neutral", icon: "hourglass_top" },
    date: "18 août",
  },
  {
    id: "3",
    job_title: "Ingénieur DevOps",
    company: "Kelvin Systems",
    contract: "CDI",
    status: { label: "Relancée", tone: "warning", icon: "send" },
    date: "15 août",
  },
  {
    id: "4",
    job_title: "Data Analyst",
    company: "Solstice Analytics",
    contract: "CDI",
    status: { label: "Refusée", tone: "danger", icon: "do_not_disturb_on" },
    date: "08 août",
  },
];

const COLUMNS: Column<DemoRow, "poste" | "entreprise" | "statut" | "date">[] = [
  { key: "poste", header: "Poste", sort_key: "poste", render: (row) => row.job_title },
  {
    key: "entreprise",
    header: "Entreprise",
    sort_key: "entreprise",
    render: (row) => <span className="text-ink-muted">{row.company}</span>,
  },
  { key: "contrat", header: "Contrat", grow: 0.7, render: (row) => row.contract },
  {
    key: "statut",
    header: "Statut",
    sort_key: "statut",
    grow: 1,
    render: (row) => (
      <StatusPill tone={row.status.tone} icon={row.status.icon}>
        {row.status.label}
      </StatusPill>
    ),
  },
  {
    key: "date",
    header: "Envoyée",
    sort_key: "date",
    grow: 0.7,
    numeric: true,
    render: (row) => row.date,
  },
];

const THEMES: ThemePref[] = ["light", "dark", "system"];

export function DesignGallery() {
  const [modal, setModal] = useState(false);
  const [confirm, setConfirm] = useState(false);
  const [page, setPage] = useState(1);
  const [sort_key, setSortKey] = useState<"poste" | "entreprise" | "statut" | "date">("date");
  const { theme, setTheme, notify } = useUiStore();

  return (
    <div className="flex h-full flex-col">
      <PageHeader
        icon="palette"
        title="Design system"
        subtitle="Planche de vérification"
        secondary={
          <div className="flex items-center gap-1 rounded-button border border-line bg-surface p-0.5">
            {THEMES.map((value) => (
              <button
                key={value}
                type="button"
                onClick={() => setTheme(value)}
                className={`rounded-[6px] px-2.5 py-1 text-meta transition-colors duration-150 ${
                  theme === value ? "bg-accent-tint text-accent" : "text-ink-muted hover:text-ink"
                }`}
              >
                {value}
              </button>
            ))}
          </div>
        }
        primary={
          <Button variant="primary" icon="add" onClick={() => setModal(true)}>
            Ouvrir une modale
          </Button>
        }
      />

      <div className="flex flex-col gap-7 overflow-y-auto p-7">
        <Section title="Boutons">
          <div className="flex flex-wrap items-center gap-2">
            <Button variant="primary" icon="add">Action primaire</Button>
            <Button variant="secondary" icon="download">Secondaire</Button>
            <Button variant="ghost" icon="filter_alt">FantMe</Button>
            <Button variant="danger" icon="delete" onClick={() => setConfirm(true)}>
              Destructive
            </Button>
            <Button variant="primary" disabled>DSactiv</Button>
          </div>
        </Section>

        <Section title="Indicateurs">
          <div className="grid grid-cols-4 gap-4">
            <StatCard icon="work" label="Candidatures" value="15" delta="+3 ce mois" deltaTone="success" />
            <StatCard icon="event" label="Entretiens" value="4" delta="2 à venir" deltaTone="accent" />
            <StatCard icon="send" label="Relances" value="7" delta="1 en retard" deltaTone="warning" />
            <StatCard icon="percent" label="Taux de réponse" value="46 %" />
          </div>
        </Section>

        <Section title="Statuts">
          <div className="flex flex-wrap items-center gap-2">
            <StatusPill tone="success" icon="event_available">Entretien</StatusPill>
            <StatusPill tone="neutral" icon="hourglass_top">En attente</StatusPill>
            <StatusPill tone="warning" icon="send">Relancée</StatusPill>
            <StatusPill tone="danger" icon="do_not_disturb_on">Refusée</StatusPill>
            <StatusPill tone="accent" icon="auto_awesome">Généré par IA</StatusPill>
          </div>
        </Section>

        <Section title="Tableau, tri et pagination">
          <div className="overflow-hidden rounded-card border border-line">
            <DataTable
              columns={COLUMNS}
              rows={ROWS}
              row_key={(row) => row.id}
              sort={{ key: sort_key, direction: "desc" }}
              onSortChange={setSortKey}
              onRowClick={() => notify({ tone: "info", title: "Ligne ouverte" })}
            />
            <Pager page={page} page_size={4} total={15} label="candidatures" onPageChange={setPage} />
          </div>
        </Section>

        <Section title="Formulaire">
          <div className="grid grid-cols-2 gap-4 rounded-card border border-line bg-surface p-4">
            <FormField label="Poste" required>
              {(props) => <TextInput {...props} defaultValue="Développeur Frontend" />}
            </FormField>
            <FormField label="Contrat">
              {(props) => (
                <Select {...props} defaultValue="CDI">
                  <option>CDI</option>
                  <option>CDD</option>
                  <option>Freelance</option>
                </Select>
              )}
            </FormField>
            <FormField label="Date d'envoi" required error="Date invalide — format attendu JJ-MM-AAAA.">
              {(props) => <TextInput {...props} defaultValue="32-08-2026" invalid />}
            </FormField>
            <FormField label="Lien de l'offre" help="Facultatif, en HTTP ou HTTPS.">
              {(props) => <TextInput {...props} placeholder="https://…" />}
            </FormField>
            <div className="col-span-2">
              <FormField label="Notes">
                {(props) => <TextArea {...props} placeholder="Contexte, personne rencontrée…" />}
              </FormField>
            </div>
          </div>
        </Section>

        <Section title="États">
          <div className="grid grid-cols-2 gap-4">
            <div className="overflow-hidden rounded-card border border-line bg-surface">
              <SkeletonRows rows={3} columns={4} />
            </div>
            <div className="rounded-card border border-line bg-surface">
              <EmptyState
                title="Aucune candidature"
                description="Créez votre première candidature pour lancer le suivi."
                action={<Button variant="primary" icon="add">Nouvelle candidature</Button>}
              />
            </div>
            <div className="col-span-2">
              <ErrorBanner
                message="Vérifiez votre clé API et l'endpoint dans Réglages → Intelligence artificielle."
                onRetry={() => notify({ tone: "info", title: "Nouvelle tentative" })}
              />
            </div>
          </div>
        </Section>

        <Section title="Chronologie">
          <div className="rounded-card border border-line bg-surface p-4">
            <TimelineList
              entries={[
                { id: "a", title: "Entretien planifié", detail: "Visio 45 min avec Camille Rivet", date: "25 août", tone: "success" },
                { id: "b", title: "Réponse reçue", detail: "Invitation à un premier échange", date: "20 août", tone: "accent" },
                { id: "c", title: "Relance envoyée", detail: "Email de suivi, ton formel", date: "14 août", tone: "warning" },
              ]}
            />
          </div>
        </Section>

        <Section title="Notifications">
          <div className="flex flex-wrap gap-2">
            <Button icon="check_circle" onClick={() => notify({ tone: "success", title: "Candidature enregistrée", detail: "Ajoutée au suivi." })}>
              Succès
            </Button>
            <Button icon="error" onClick={() => notify({ tone: "error", title: "Enregistrement impossible", detail: "Le poste est requis." })}>
              Erreur
            </Button>
            <Button icon="info" onClick={() => notify({ tone: "info", title: "Analyse en cours" })}>
              Information
            </Button>
          </div>
        </Section>
      </div>

      <ModalHost
        open={modal}
        icon="work"
        title="Nouvelle candidature"
        subtitle="Renseignez le poste et l'entreprise visés"
        footer_note="Les dates sont saisies au format JJ-MM-AAAA."
        onClose={() => setModal(false)}
        onSubmit={() => {
          setModal(false);
          notify({ tone: "success", title: "Candidature enregistrée" });
        }}
      >
        <div className="grid grid-cols-2 gap-4">
          <div className="col-span-2">
            <FormField label="Poste" required>
              {(props) => <TextInput {...props} placeholder="Développeur Frontend" />}
            </FormField>
          </div>
          <FormField label="Contrat">
            {(props) => (
              <Select {...props}>
                <option>CDI</option>
                <option>CDD</option>
              </Select>
            )}
          </FormField>
          <FormField label="Date d'envoi" required>
            {(props) => <TextInput {...props} defaultValue="22-08-2026" />}
          </FormField>
        </div>
      </ModalHost>

      <ConfirmDialog
        open={confirm}
        title="Supprimer cette candidature ?"
        description="« Développeur Frontend » chez Nova Digital sera définitivement supprimée, ainsi que l'entretien et la relance rattachés. Cette action est irréversible."
        note="L'entreprise et le contact associés sont conservés."
        onCancel={() => setConfirm(false)}
        onConfirm={() => {
          setConfirm(false);
          notify({ tone: "success", title: "Candidature supprimée" });
        }}
      />
    </div>
  );
}

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="flex flex-col gap-3">
      <h2 className="text-eyebrow uppercase text-ink-faint">{title}</h2>
      {children}
    </section>
  );
}
