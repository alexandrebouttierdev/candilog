import { useState } from "react";
import type { KeyboardEvent } from "react";
import { AppError } from "@/shared/types/app-error";
import type { Identity, Profile } from "@/shared/types/generated/profile";
import { Button, ConfirmDialog, ErrorBanner, Kbd, Skeleton, StatusGlyph } from "@/shared/ui";
import type { GlyphTone } from "@/shared/ui";
import { useChrome } from "@/shared/lib/chrome";
import { useShortcut } from "@/shared/hooks/useShortcut";
import { cn } from "@/shared/lib/cn";
import { useProfilePhoto, useProfileViewModel } from "../../viewmodel/useProfileViewModel";
import { ProfileSectionModal, type ProfileSection } from "../components/ProfileSectionModal";
import { ProfileImportModal } from "../components/ProfileImportModal";

/** Sections de la colonne de gauche (`screens/11-profile.png`). */
type SectionKey =
  | "identity"
  | "contact"
  | "headline"
  | "experiences"
  | "education"
  | "skills"
  | "projects"
  | "languages"
  | "certifications"
  | "interests"
  | "online"
  | "preferences";

type ListKey = "experiences" | "education" | "skills" | "projects" | "languages" | "certifications" | "interests";

interface FieldRow {
  readonly label: string;
  readonly value: string | null;
  readonly multiline?: boolean;
}

interface SectionMeta {
  readonly label: string;
  /** Formulaire qui édite la section. */
  readonly editor: ProfileSection;
  /** Phrase d'accroche sous le titre. */
  readonly intro: string;
  /** Libellé de l'ajout, pour les sections en liste. */
  readonly add?: string;
}

const SECTIONS: Record<SectionKey, SectionMeta> = {
  identity: { label: "Identité", editor: "identity", intro: "Ce qui figure en tête de vos CV et de vos lettres." },
  contact: { label: "Contact", editor: "identity", intro: "Comment un recruteur vous joint." },
  headline: { label: "Titre & accroche", editor: "objective", intro: "Le titre et le résumé placés sous votre nom." },
  experiences: { label: "Expériences", editor: "experiences", intro: "L'IA choisit lesquelles mettre en avant selon l'offre visée.", add: "Ajouter une expérience" },
  education: { label: "Formations", editor: "education", intro: "Diplômes et parcours de formation.", add: "Ajouter une formation" },
  skills: { label: "Compétences", editor: "skills", intro: "Les savoir-faire que le générateur rapproche des exigences de l'offre.", add: "Ajouter des compétences" },
  projects: { label: "Projets", editor: "projects", intro: "Réalisations personnelles ou professionnelles à valoriser.", add: "Ajouter un projet" },
  languages: { label: "Langues", editor: "languages", intro: "Votre niveau de pratique, langue par langue.", add: "Ajouter une langue" },
  certifications: { label: "Certifications", editor: "certifications", intro: "Qualifications reconnues.", add: "Ajouter une certification" },
  interests: { label: "Centres d'intérêt", editor: "interests", intro: "Facultatif — ce qui vous distingue en dehors du travail.", add: "Ajouter des centres d'intérêt" },
  online: { label: "Présence en ligne", editor: "online", intro: "Liens professionnels repris dans l'en-tête du CV." },
  preferences: { label: "Préférences", editor: "objective", intro: "Ce que vous recherchez : contrats et disponibilité." },
};

const ORDER: readonly SectionKey[] = [
  "identity", "contact", "headline", "experiences", "education", "skills", "projects",
  "languages", "certifications", "interests", "online", "preferences",
];

function fieldsOf(key: SectionKey, identity: Identity): FieldRow[] {
  switch (key) {
    case "identity":
      return [
        { label: "Prénom", value: identity.first_name || null },
        { label: "Nom", value: identity.name || null },
        { label: "Date de naissance", value: identity.birth_date },
        { label: "Âge", value: identity.age == null ? null : `${identity.age} ans` },
      ];
    case "contact":
      return [
        { label: "Courriel", value: identity.email || null },
        { label: "Téléphone", value: identity.phone },
        { label: "Adresse", value: identity.address },
        { label: "Ville", value: identity.city },
      ];
    case "headline":
      return [
        { label: "Titre", value: identity.title },
        { label: "Accroche", value: identity.resume, multiline: true },
      ];
    case "online":
      return [
        { label: "LinkedIn", value: identity.linkedin },
        { label: "GitHub", value: identity.github },
        { label: "Site", value: identity.website },
      ];
    case "preferences":
      return [
        { label: "Contrats", value: identity.desired_contracts },
        { label: "Disponibilité", value: identity.availability },
      ];
    default:
      return [];
  }
}

function isList(key: SectionKey): key is ListKey {
  return ["experiences", "education", "skills", "projects", "languages", "certifications", "interests"].includes(key);
}

/** État d'une section : plein, partiel ou vide, avec son décompte. */
function statusOf(key: SectionKey, profile: Profile): { tone: GlyphTone; count: string; complete: boolean } {
  if (isList(key)) {
    const count = profile[key].length;
    return { tone: count > 0 ? "g" : "n", count: String(count), complete: count > 0 };
  }
  const rows = fieldsOf(key, profile.identity);
  const filled = rows.filter((row) => row.value).length;
  return {
    tone: filled === rows.length ? "g" : filled > 0 ? "a" : "n",
    count: `${filled}/${rows.length}`,
    complete: filled === rows.length,
  };
}

/** Entrées affichables d'une section en liste. */
function entriesOf(key: ListKey, profile: Profile): Array<{ title: string; sub: string | null; period: string | null; body: string | null }> {
  switch (key) {
    case "experiences":
      return profile.experiences.map((item) => ({
        title: item.title,
        sub: [item.company, item.location].filter(Boolean).join(" · ") || null,
        period: `${item.start_date.slice(0, 4)} → ${item.current ? "auj." : (item.end_date?.slice(0, 4) ?? "?")}`,
        body: item.description,
      }));
    case "education":
      return profile.education.map((item) => ({
        title: item.degree,
        sub: [item.school, item.location].filter(Boolean).join(" · ") || null,
        period: [item.start_date, item.end_date].filter(Boolean).join(" → ") || null,
        body: item.description,
      }));
    case "skills":
      return profile.skills.map((item) => ({ title: item.name, sub: null, period: null, body: item.description }));
    case "projects":
      return profile.projects.map((item) => ({ title: item.name, sub: item.technologies, period: null, body: item.description }));
    case "languages":
      return profile.languages.map((item) => ({ title: item.name, sub: item.level, period: null, body: null }));
    case "certifications":
      return profile.certifications.map((item) => ({
        title: item.name,
        sub: item.issuer,
        period: item.date,
        body: item.description,
      }));
    case "interests":
      return profile.interests.map((item) => ({ title: item.name, sub: null, period: null, body: null }));
  }
}

/** Retire l'entrée `index` d'une section en liste. */
function without(profile: Profile, key: ListKey, index: number): Profile {
  switch (key) {
    case "experiences":
      return { ...profile, experiences: profile.experiences.filter((_, at) => at !== index) };
    case "education":
      return { ...profile, education: profile.education.filter((_, at) => at !== index) };
    case "skills":
      return { ...profile, skills: profile.skills.filter((_, at) => at !== index) };
    case "projects":
      return { ...profile, projects: profile.projects.filter((_, at) => at !== index) };
    case "languages":
      return { ...profile, languages: profile.languages.filter((_, at) => at !== index) };
    case "certifications":
      return { ...profile, certifications: profile.certifications.filter((_, at) => at !== index) };
    case "interests":
      return { ...profile, interests: profile.interests.filter((_, at) => at !== index) };
  }
}

/** « il y a 2 min », « il y a 3 h », « le 12-09 ». */
function savedAgo(iso: string, now = new Date()): string {
  const minutes = Math.round((now.getTime() - new Date(iso).getTime()) / 60_000);
  if (minutes < 1) return "à l'instant";
  if (minutes < 60) return `il y a ${minutes} min`;
  if (minutes < 60 * 24) return `il y a ${Math.round(minutes / 60)} h`;
  return `le ${iso.slice(8, 10)}-${iso.slice(5, 7)}`;
}

/**
 * Profil (`screens/11-profile.png`) : colonne des sections (complétude, état, décompte)
 * et contenu de la section choisie. Le profil est la source de vérité des CV et des
 * lettres ; chaque section s'édite dans son formulaire validé.
 *
 * Contrat clavier : `↑ ↓` changent de section, `⏎` modifie, `⌘N` ajoute une entrée.
 */
export function ProfilePage() {
  const vm = useProfileViewModel();
  const photo = useProfilePhoto().data ?? null;
  const [active, setActive] = useState<SectionKey>("identity");
  const [editor, setEditor] = useState<ProfileSection | null>(null);
  const [importOpen, setImportOpen] = useState(false);
  const [resetOpen, setResetOpen] = useState(false);
  const [pendingRemove, setPendingRemove] = useState<{ key: ListKey; index: number; title: string } | null>(null);

  const profile = vm.data?.profile ?? null;
  const meta = SECTIONS[active];
  const incomplete = profile ? ORDER.filter((key) => !statusOf(key, profile).complete).length : 0;

  useShortcut("mod+n", () => setEditor(meta.editor), { enabled: profile !== null && isList(active) });

  useChrome({
    crumb: meta.label,
    ...(vm.data?.updated_at ? { aside: `Enregistré ${savedAgo(vm.data.updated_at)}` } : {}),
    status: vm.data
      ? `profil ${vm.data.completion} % · ${incomplete} section${incomplete > 1 ? "s" : ""} incomplète${incomplete > 1 ? "s" : ""}`
      : "",
    keys: [
      { label: "Modifier", shortcut: "enter" },
      ...(isList(active) ? [{ label: "Ajouter", shortcut: "mod+n" }] : []),
    ],
  });

  const onNavKey = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === "Enter") {
      // `⏎` sur une section la modifie, comme le rappelle la barre d'état.
      event.preventDefault();
      setEditor(meta.editor);
      return;
    }
    if (event.key !== "ArrowDown" && event.key !== "ArrowUp") return;
    event.preventDefault();
    const index = ORDER.indexOf(active);
    const next = ORDER[Math.min(Math.max(index + (event.key === "ArrowDown" ? 1 : -1), 0), ORDER.length - 1)]!;
    setActive(next);
    document.getElementById(`section-${next}`)?.focus();
  };

  if (vm.isLoading) {
    return (
      <div role="status" aria-label="Chargement du profil" className="flex h-full">
        <div className="w-[210px] flex-none border-r border-bd-soft p-3">
          {[1, 2, 3, 4, 5, 6].map((index) => (
            <Skeleton key={index} index={index} className="mb-3 h-3" />
          ))}
        </div>
        <div className="flex-1 p-6">
          <Skeleton className="h-6 w-48" />
        </div>
      </div>
    );
  }

  if (vm.error || !vm.data || !profile) {
    return (
      <div className="p-6">
        <ErrorBanner
          message={vm.error instanceof AppError ? vm.error.message : "Le profil n'a pas pu être chargé."}
          onRetry={vm.reload}
        />
      </div>
    );
  }

  const completion = vm.data.completion;
  const status = statusOf(active, profile);

  return (
    <div className="flex h-full min-h-0">
      <nav aria-label="Profil" className="flex w-[210px] flex-none flex-col border-r border-bd-soft">
        <div className="px-3.5 pt-3.5 pb-2.5">
          <p className="flex items-baseline gap-1.5">
            <span className="serif-title text-[19px] leading-none text-tx">{completion} %</span>
            <span className="text-sub text-tx-5">complété</span>
          </p>
          <div aria-hidden className="mt-2 flex gap-0.5">
            {ORDER.map((key) => {
              const tone = statusOf(key, profile).tone;
              return (
                <span
                  key={key}
                  className={cn("h-[3px] flex-1 rounded-r2", tone === "g" ? "bg-st-g" : tone === "a" ? "bg-st-a" : "bg-chip")}
                />
              );
            })}
          </div>
        </div>
        <div
          role="tablist"
          aria-label="Sections du profil"
          aria-orientation="vertical"
          onKeyDown={onNavKey}
          className="flex-1 overflow-y-auto px-2"
        >
          {ORDER.map((key) => {
            const state = statusOf(key, profile);
            const selected = key === active;
            return (
              <button
                key={key}
                id={`section-${key}`}
                type="button"
                role="tab"
                aria-selected={selected}
                aria-controls="section-contenu"
                tabIndex={selected ? 0 : -1}
                onClick={() => setActive(key)}
                className={cn(
                  "flex h-[27px] w-full items-center gap-[9px] rounded-r6 px-2 text-left text-ui",
                  selected ? "bg-elev font-medium text-tx" : "text-tx-2 hover:bg-hover",
                )}
              >
                {state.tone === "g" ? (
                  // Section complète : disque plein, le demi-disque restant réservé au partiel.
                  <span aria-hidden className="size-[9px] flex-none rounded-full bg-st-g" />
                ) : (
                  <StatusGlyph tone={state.tone} small />
                )}
                <span className="truncate">{SECTIONS[key].label}</span>
                <span
                  className={cn(
                    "ml-auto font-mono text-[10px]",
                    state.tone === "a" ? "text-st-a" : state.tone === "n" ? "text-tx-6" : "text-tx-5",
                  )}
                >
                  {state.count}
                </span>
              </button>
            );
          })}
        </div>
        <div className="border-t border-bd-soft px-3.5 pt-3 pb-3.5">
          <Button size="compact" className="w-full" onClick={() => setImportOpen(true)}>
            Importer un CV
          </Button>
          <p className="mt-2 text-tiny leading-[1.45] text-tx-5">
            Compare avec votre profil avant d'écraser quoi que ce soit.
          </p>
          <button
            type="button"
            disabled={vm.isResetting}
            onClick={() => setResetOpen(true)}
            className="mt-3 text-tiny text-st-c hover:underline disabled:opacity-50"
          >
            Réinitialiser mon profil
          </button>
        </div>
      </nav>

      <section
        id="section-contenu"
        role="tabpanel"
        aria-labelledby={`section-${active}`}
        className="min-w-0 flex-1 overflow-y-auto px-[22px] pt-[18px] pb-8"
      >
        <div className="mx-auto max-w-[760px]">
          <div className="flex items-start gap-3 border-b border-bd-soft pb-3.5">
            <div className="min-w-0 flex-1">
              <h1 className="serif-title text-[23px] leading-tight text-tx">{meta.label}</h1>
              <p className="mt-1 text-small text-tx-4">{meta.intro}</p>
            </div>
            {status.complete ? (
              <span className="inline-flex h-[22px] flex-none items-center rounded-r6 bg-tint-g-bg px-2 text-small text-tint-g-tx">
                Complet
              </span>
            ) : null}
            <Button size="compact" onClick={() => setEditor(meta.editor)}>
              Modifier
              <Kbd shortcut="enter" tone="ghost" decorative />
            </Button>
          </div>

          {active === "identity" ? (
            <PhotoBlock
              name={`${profile.identity.first_name} ${profile.identity.name}`.trim()}
              title={profile.identity.title}
              photo={photo}
              hasPhoto={profile.photo !== null}
              busy={vm.isPhotoBusy}
              onPick={() => void vm.setPhoto()}
              onRemove={() => void vm.removePhoto()}
            />
          ) : null}

          {isList(active) ? (
            <EntryList
              entries={entriesOf(active, profile)}
              chips={active === "skills" || active === "interests"}
              add={meta.add ?? "Ajouter"}
              onAdd={() => setEditor(meta.editor)}
              onEdit={() => setEditor(meta.editor)}
              onRemove={(index, title) => setPendingRemove({ key: active, index, title })}
            />
          ) : (
            <dl className="mt-3 grid grid-cols-[160px_minmax(0,1fr)] gap-x-3 gap-y-2.5 text-row">
              {fieldsOf(active, profile.identity).map((row) => (
                <Field key={row.label} row={row} onEdit={() => setEditor(meta.editor)} />
              ))}
            </dl>
          )}
        </div>
      </section>

      {editor ? (
        <ProfileSectionModal
          key={editor}
          section={editor}
          profile={profile}
          busy={vm.isSaving}
          onClose={() => setEditor(null)}
          onSubmit={vm.save}
        />
      ) : null}

      <ConfirmDialog
        open={pendingRemove !== null}
        title="Retirer cette entrée ?"
        description={`« ${pendingRemove?.title ?? ""} » disparaît de votre profil. Les CV déjà enregistrés ne changent pas.`}
        confirmLabel="Retirer"
        busy={vm.isSaving}
        onCancel={() => setPendingRemove(null)}
        onConfirm={() => {
          const target = pendingRemove;
          setPendingRemove(null);
          if (target) vm.save(without(profile, target.key, target.index)).catch(() => undefined);
        }}
      />

      <ConfirmDialog
        open={resetOpen}
        title="Réinitialiser le profil ?"
        description="Toutes les informations de votre profil seront supprimées, photo comprise."
        note="Vos candidatures, entreprises, contacts, entretiens et autres données ne sont pas modifiés."
        confirmLabel="Réinitialiser"
        busy={vm.isResetting}
        onCancel={() => setResetOpen(false)}
        onConfirm={() => {
          setResetOpen(false);
          vm.reset().catch(() => undefined);
        }}
      />

      {importOpen ? (
        <ProfileImportModal open busy={vm.isSaving} onClose={() => setImportOpen(false)} onApply={vm.applyImport} />
      ) : null}
    </div>
  );
}

function Field({ row, onEdit }: { row: FieldRow; onEdit: () => void }) {
  return (
    <>
      <dt className="pt-px text-small text-tx-5">{row.label}</dt>
      <dd className={cn("min-w-0", row.multiline ? "leading-[1.55] whitespace-pre-wrap" : "truncate")}>
        {row.value ? (
          <span className="text-tx-2">{row.value}</span>
        ) : (
          <button type="button" onClick={onEdit} className="text-small text-tx-6 hover:text-ac-tx">
            Non renseigné — compléter
          </button>
        )}
      </dd>
    </>
  );
}

function EntryList({
  entries,
  chips,
  add,
  onAdd,
  onEdit,
  onRemove,
}: {
  entries: Array<{ title: string; sub: string | null; period: string | null; body: string | null }>;
  chips: boolean;
  add: string;
  onAdd: () => void;
  onEdit: () => void;
  onRemove: (index: number, title: string) => void;
}) {
  const addButton = (
    <button
      type="button"
      onClick={onAdd}
      className="mt-2 flex h-8 w-full items-center gap-2 rounded-r6 px-1 text-left text-small text-tx-4 hover:text-tx-2"
    >
      <span aria-hidden>+</span>
      {add}
      <Kbd shortcut="mod+n" tone="ghost" decorative className="ml-auto" />
    </button>
  );

  if (entries.length === 0) {
    return (
      <div className="mt-4">
        <p className="text-small text-tx-5">Rien pour l'instant.</p>
        {addButton}
      </div>
    );
  }

  if (chips) {
    return (
      <div className="mt-4">
        <ul className="flex flex-wrap gap-1.5">
          {entries.map((entry, index) => (
            <li
              key={`${entry.title}-${index}`}
              title={entry.body ?? undefined}
              className="inline-flex h-6 items-center gap-1.5 rounded-r6 bg-chip pr-1 pl-2 text-small text-tx-2"
            >
              {entry.title}
              <button
                type="button"
                aria-label={`Retirer ${entry.title}`}
                onClick={() => onRemove(index, entry.title)}
                className="flex size-4 items-center justify-center rounded-r4 text-[10px] text-tx-5 hover:bg-hover hover:text-tx-2"
              >
                ✕
              </button>
            </li>
          ))}
        </ul>
        {addButton}
      </div>
    );
  }

  return (
    <div className="mt-3">
      <ul className="flex flex-col gap-3.5">
        {entries.map((entry, index) => (
          <li
            key={`${entry.title}-${index}`}
            className={cn("group border-l-2 py-0.5 pl-2.5", index === 0 ? "border-ac" : "border-bd-menu")}
          >
            <div className="flex items-baseline gap-3">
              <button type="button" onClick={onEdit} className="min-w-0 truncate text-left text-row font-medium text-tx hover:underline">
                {entry.title}
              </button>
              {entry.period ? (
                <span className="ml-auto flex-none font-mono text-caps text-tx-5">{entry.period}</span>
              ) : null}
              <button
                type="button"
                aria-label={`Retirer ${entry.title}`}
                onClick={() => onRemove(index, entry.title)}
                className={cn("flex-none text-small text-tx-6 hover:text-st-c", !entry.period && "ml-auto")}
              >
                ×
              </button>
            </div>
            {entry.sub ? <p className="text-sub text-tx-4">{entry.sub}</p> : null}
            {entry.body ? <p className="mt-1 text-small leading-[1.55] text-tx-3">{entry.body}</p> : null}
          </li>
        ))}
      </ul>
      {addButton}
    </div>
  );
}

function PhotoBlock({
  name,
  title,
  photo,
  hasPhoto,
  busy,
  onPick,
  onRemove,
}: {
  name: string;
  title: string | null;
  photo: string | null;
  hasPhoto: boolean;
  busy: boolean;
  onPick: () => void;
  onRemove: () => void;
}) {
  const initials =
    name
      .split(/\s+/)
      .filter(Boolean)
      .slice(0, 2)
      .map((word) => word[0]?.toUpperCase())
      .join("") || "?";
  return (
    <div className="mt-4 flex items-center gap-3.5 rounded-r9 bg-group px-3.5 py-3">
      {photo ? (
        <img src={photo} alt="Photo de profil" className="size-14 flex-none rounded-field object-cover" />
      ) : (
        <span aria-hidden className="flex size-14 flex-none items-center justify-center rounded-field bg-av1 text-lead font-medium text-av-tx">
          {initials}
        </span>
      )}
      <div className="min-w-0 flex-1">
        <h2 className="truncate text-entry font-medium text-tx">{name || "Votre nom"}</h2>
        <p className="truncate text-sub text-tx-4">{title ?? "Titre non renseigné"}</p>
      </div>
      <div className="flex flex-none gap-1.5">
        <Button size="compact" disabled={busy} onClick={onPick}>
          {hasPhoto ? "Remplacer la photo" : "Ajouter une photo"}
        </Button>
        {hasPhoto ? (
          <Button variant="ghost" size="compact" disabled={busy} onClick={onRemove}>
            Supprimer la photo
          </Button>
        ) : null}
      </div>
    </div>
  );
}
