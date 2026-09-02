import { useEffect, useRef, useState } from "react";
import type {
  ResumeCertificationBlock,
  ResumeEducationBlock,
  ResumeExperienceBlock,
  ResumeIdentity,
  ResumeLanguageBlock,
  ResumeProjectBlock,
  ResumeSkillGroup,
  ResumeWorkspace,
} from "@/shared/types/generated/documents";
import { safeResumeUrl, type ResumeField } from "../../model/resumeWorkspace";
import { ResumeEditableText } from "./ResumeEditableText";

type ResumeFieldChange = (field: ResumeField, value: string) => void;

/** Paliers de densité du template fourni : le premier aère un profil court, le dernier est
 *  le seuil lisible minimal. L'ordre est celui essayé par la mesure de débordement. */
const DENSITY_STEPS: { fs: number; sp: number }[] = [
  { fs: 1.04, sp: 1.35 },
  { fs: 1, sp: 1 },
  { fs: 1, sp: 0.82 },
  { fs: 0.96, sp: 0.72 },
  { fs: 0.92, sp: 0.62 },
];

// La colonne d'étiquettes porte le plus long libellé du gabarit (« PROFESSIONNELLES ») à
// son interlettrage réel : à 104 px il en sortait, et l'aperçu ne montrait plus la
// colonne qu'exporte `resume_pdf.rs` (`LABEL_W`).
const GRID = "grid grid-cols-[116px_1fr] items-start gap-[18px]";
const LABEL = "resume-mono m-0 text-[calc(9.2px*var(--resume-fs))] font-medium uppercase leading-[1.4] tracking-[0.11em] text-[var(--resume-accent)]";
const META = "resume-mono whitespace-nowrap text-[calc(9.5px*var(--resume-fs))] tabular-nums tracking-[0.01em] text-[var(--resume-subtle)]";
const SEPARATOR = "text-[var(--resume-subtle)]";

/**
 * Papier A4 du CV, éditable en place, porté du template fourni.
 *
 * `editable={false}` sert la bibliothèque et l'aperçu en lecture ; `editable` sert
 * l'éditeur. Dans les deux cas la hiérarchie et les jetons `resume-*` restent identiques —
 * seul `ResumeEditableText` change de rendu, jamais la disposition.
 */
export function ResumePaper({
  workspace,
  editable,
  onChange,
  onOverflowChange,
}: {
  workspace: ResumeWorkspace;
  editable: boolean;
  onChange: ResumeFieldChange;
  onOverflowChange?: (overflow: boolean) => void;
}) {
  const { document } = workspace;
  const paperRef = useRef<HTMLElement | null>(null);
  const contentRef = useRef<HTMLDivElement | null>(null);
  const [overflow, setOverflow] = useState(false);

  useEffect(() => {
    const page = paperRef.current;
    const content = contentRef.current;
    if (!page || !content) return;

    let arrete = false;
    const mesurer = () => {
      if (arrete) return;
      const depasse = ajusterDensite(page, content);
      setOverflow(depasse);
      onOverflowChange?.(depasse);
    };

    const polices = typeof globalThis.document !== "undefined" ? globalThis.document.fonts : undefined;
    const pret = polices ? polices.ready : Promise.resolve();
    void pret.then(mesurer);

    let observateur: ResizeObserver | undefined;
    if (typeof ResizeObserver !== "undefined") {
      observateur = new ResizeObserver(mesurer);
      observateur.observe(page);
    }

    return () => {
      arrete = true;
      observateur?.disconnect();
    };
  }, [workspace, editable, onOverflowChange]);

  return (
    <>
      {overflow ? (
        <p className="resume-overflow-warning" data-print-hide>
          Contenu trop long pour une page A4 : espacements et typographie sont déjà au seuil
          minimal. Raccourcis le profil ou les missions les moins récentes.
        </p>
      ) : null}
      <article ref={paperRef} aria-label="CV" className="resume-paper">
        <div ref={contentRef} className="resume-content">
          <ResumeHeader identity={document.identity} editable={editable} onChange={onChange} />
          <main className="flex flex-col gap-[calc(13px*var(--resume-sp))]">
            <ProfileSection profile={document.profile} editable={editable} onChange={onChange} />
            <ExperiencesSection experiences={document.experiences} editable={editable} onChange={onChange} />
            <ProjectsSection projects={document.projects} editable={editable} onChange={onChange} />
            <SkillsSection groups={document.skill_groups} editable={editable} onChange={onChange} />
            <EducationSection education={document.education} editable={editable} onChange={onChange} />
            <CertificationsSection certifications={document.certifications} editable={editable} onChange={onChange} />
            <LanguagesSection languages={document.languages} editable={editable} onChange={onChange} />
          </main>
        </div>
      </article>
    </>
  );
}

/**
 * Applique le palier le moins dense, puis compacte tant que le dernier bloc dépasse la
 * hauteur imprimable ; ré-aère si un palier plus généreux tient encore. Reproduit `fit()`
 * du template fourni, en comparant `scrollHeight`/`clientHeight` via les rectangles réels.
 */
function ajusterDensite(page: HTMLElement, content: HTMLElement): boolean {
  let index = 1;
  const appliquer = (position: number) => {
    page.style.setProperty("--resume-fs", String(DENSITY_STEPS[position]?.fs ?? 1));
    page.style.setProperty("--resume-sp", String(DENSITY_STEPS[position]?.sp ?? 1));
  };
  appliquer(index);

  const dernier = content.lastElementChild;
  if (!dernier) return false;

  const utilise = () => {
    const cadre = content.getBoundingClientRect();
    const bloc = dernier.getBoundingClientRect();
    if (cadre.height === 0) return 0;
    return (bloc.bottom - cadre.top) / cadre.height;
  };

  while (utilise() > 1.001 && index < DENSITY_STEPS.length - 1) appliquer(++index);
  while (utilise() < 0.84 && index > 0) {
    appliquer(index - 1);
    if (utilise() > 1.001) {
      appliquer(index);
      break;
    }
    index--;
  }
  return utilise() > 1.001;
}

function ResumeHeader({
  identity,
  editable,
  onChange,
}: {
  identity: ResumeIdentity;
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  const contacts: { label: string; field: Extract<ResumeField, { type: "identity" }>["field"]; value: string | null }[] = [
    { label: "Ville", field: "city", value: identity.city },
    { label: "Téléphone", field: "phone", value: identity.phone },
  ];
  const liens: { label: string; field: Extract<ResumeField, { type: "identity" }>["field"]; value: string | null }[] = [
    { label: "Site web", field: "website", value: identity.website },
    { label: "LinkedIn", field: "linkedin", value: identity.linkedin },
    { label: "GitHub", field: "github", value: identity.github },
  ];

  return (
    <header className="flex flex-col gap-[calc(10px*var(--resume-sp))]">
      <div className="flex flex-col gap-[calc(3px*var(--resume-sp))]">
        <ResumeEditableText
          tag="h1"
          editable={editable}
          label="Nom complet"
          value={identity.full_name}
          className="m-0 text-[calc(31px*var(--resume-fs))] font-semibold leading-[1.02] tracking-[-0.028em] text-[var(--resume-ink)]"
          onChange={(value) => onChange({ type: "identity", field: "full_name" }, value)}
        />
        <ResumeEditableText
          tag="p"
          editable={editable}
          label="Titre professionnel"
          value={identity.title}
          className="resume-mono m-0 mt-[calc(3px*var(--resume-sp))] text-[calc(10.4px*var(--resume-fs))] font-medium uppercase tracking-[0.15em] text-[var(--resume-accent)]"
          onChange={(value) => onChange({ type: "identity", field: "title" }, value)}
        />
        {identity.headline ? (
          <ResumeEditableText
            tag="p"
            editable={editable}
            multiline
            label="Accroche"
            value={identity.headline}
            className="m-0 mt-[calc(2px*var(--resume-sp))] max-w-[64ch] text-[calc(11.4px*var(--resume-fs))] leading-[1.45] text-[var(--resume-body)]"
            onChange={(value) => onChange({ type: "identity", field: "headline" }, value)}
          />
        ) : null}
      </div>

      <div className="resume-mono flex flex-col gap-[calc(3.5px*var(--resume-sp))] text-[calc(10.1px*var(--resume-fs))] leading-[1.45] text-[var(--resume-muted)]">
        <p className="m-0 flex flex-wrap items-baseline gap-x-[18px] gap-y-[calc(2px*var(--resume-sp))]">
          {contacts
            .filter((entree) => entree.value !== null)
            .map((entree) => (
              <ResumeEditableText
                key={entree.field}
                tag="span"
                editable={editable}
                label={entree.label}
                value={entree.value ?? ""}
                onChange={(value) => onChange({ type: "identity", field: entree.field }, value)}
              />
            ))}
          <ResumeEditableText
            tag="span"
            editable={editable}
            label="E-mail"
            value={identity.email}
            onChange={(value) => onChange({ type: "identity", field: "email" }, value)}
          />
        </p>
        {liens.some((entree) => entree.value !== null) || identity.extra.length > 0 ? (
          <p className="m-0 flex flex-wrap items-baseline gap-x-[18px] gap-y-[calc(2px*var(--resume-sp))]">
            {liens
              .filter((entree) => entree.value !== null)
              .map((entree) =>
                editable ? (
                  <ResumeEditableText
                    key={entree.field}
                    tag="span"
                    editable
                    label={entree.label}
                    value={entree.value ?? ""}
                    onChange={(value) => onChange({ type: "identity", field: entree.field }, value)}
                  />
                ) : (
                  <SafeResumeLink
                    key={entree.field}
                    value={entree.value ?? ""}
                    className="border-b border-[var(--resume-accent-soft)] text-[var(--resume-accent)] no-underline"
                  />
                ),
              )}
            {identity.extra.map((ligne, index) => (
              <span key={`extra-${index}`}>{ligne}</span>
            ))}
          </p>
        ) : null}
      </div>
    </header>
  );
}

function ProfileSection({
  profile,
  editable,
  onChange,
}: {
  profile: string;
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  if (profile.trim() === "") return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Profil</h2>
      </div>
      <ResumeEditableText
        tag="p"
        editable={editable}
        multiline
        label="Profil"
        value={profile}
        className="m-0 text-[calc(11.6px*var(--resume-fs))] leading-[1.52] text-[var(--resume-body)]"
        onChange={(value) => onChange({ type: "profile" }, value)}
      />
    </section>
  );
}

function ExperiencesSection({
  experiences,
  editable,
  onChange,
}: {
  experiences: ResumeExperienceBlock[];
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  if (experiences.length === 0) return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Expériences professionnelles</h2>
      </div>
      <div className="flex flex-col gap-[calc(9px*var(--resume-sp))]">
        {experiences.map((experience, index) => (
          <div key={experience.id} className="flex flex-col gap-[calc(3px*var(--resume-sp))]">
            <div className="flex items-baseline justify-between gap-[14px]">
              <ResumeEditableText
                tag="h3"
                editable={editable}
                label={`Intitulé du poste ${index + 1}`}
                value={experience.title}
                className="m-0 text-[calc(12.6px*var(--resume-fs))] font-semibold tracking-[-0.006em] text-[var(--resume-ink)]"
                onChange={(value) => onChange({ type: "experience", index, field: "title" }, value)}
              />
              <ResumeEditableText
                tag="span"
                editable={editable}
                label={`Période ${index + 1}`}
                value={experience.period}
                className={META}
                onChange={(value) => onChange({ type: "experience", index, field: "period" }, value)}
              />
            </div>
            <p className="m-0 text-[calc(11.2px*var(--resume-fs))] leading-[1.4] text-[var(--resume-muted)]">
              <ResumeEditableText
                tag="span"
                editable={editable}
                label={`Entreprise ${index + 1}`}
                value={experience.company}
                className="font-medium text-[var(--resume-ink)]"
                onChange={(value) => onChange({ type: "experience", index, field: "company" }, value)}
              />
              {experience.location ? (
                <>
                  <span className={SEPARATOR}>&nbsp;·&nbsp;</span>
                  <ResumeEditableText
                    tag="span"
                    editable={editable}
                    label={`Lieu ${index + 1}`}
                    value={experience.location}
                    onChange={(value) => onChange({ type: "experience", index, field: "location" }, value)}
                  />
                </>
              ) : null}
            </p>
            <BulletList
              blockId={experience.id}
              bullets={experience.bullets}
              editable={editable}
              labelPrefix={`Réalisation ${index + 1}`}
              onChange={(item, value) => onChange({ type: "experience_bullet", index, item }, value)}
            />
          </div>
        ))}
      </div>
    </section>
  );
}

function ProjectsSection({
  projects,
  editable,
  onChange,
}: {
  projects: ResumeProjectBlock[];
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  if (projects.length === 0) return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Projets</h2>
      </div>
      <div className="flex flex-col gap-[calc(8px*var(--resume-sp))]">
        {projects.map((project, index) => (
          <div key={project.id} className="flex flex-col gap-[calc(3px*var(--resume-sp))]">
            <div className="flex items-baseline justify-between gap-[14px]">
              <ResumeEditableText
                tag="h3"
                editable={editable}
                label={`Nom du projet ${index + 1}`}
                value={project.name}
                className="m-0 text-[calc(12.2px*var(--resume-fs))] font-semibold text-[var(--resume-ink)]"
                onChange={(value) => onChange({ type: "project", index, field: "name" }, value)}
              />
              {project.url ? (
                editable ? (
                  <ResumeEditableText
                    tag="span"
                    editable
                    label={`URL du projet ${index + 1}`}
                    value={project.url}
                    className={META}
                    onChange={(value) => onChange({ type: "project", index, field: "url" }, value)}
                  />
                ) : (
                  <SafeResumeLink
                    value={project.url}
                    className={`${META} border-b border-[var(--resume-accent-soft)] text-[var(--resume-accent)] no-underline`}
                  />
                )
              ) : null}
            </div>
            {project.meta ? (
              <p className="m-0 text-[calc(11.2px*var(--resume-fs))] leading-[1.4] text-[var(--resume-muted)]">
                <ResumeEditableText
                  tag="span"
                  editable={editable}
                  label={`Contexte du projet ${index + 1}`}
                  value={project.meta}
                  onChange={(value) => onChange({ type: "project", index, field: "meta" }, value)}
                />
              </p>
            ) : null}
            <BulletList
              blockId={project.id}
              bullets={project.bullets}
              editable={editable}
              labelPrefix={`Réalisation du projet ${index + 1}`}
              onChange={(item, value) => onChange({ type: "project_bullet", index, item }, value)}
            />
          </div>
        ))}
      </div>
    </section>
  );
}

function SafeResumeLink({ value, className }: { value: string; className: string }) {
  const href = safeResumeUrl(value);
  if (href === null) return <span className={className}>{value}</span>;
  return (
    <a href={href} target="_blank" rel="noopener noreferrer" className={className}>
      {value}
    </a>
  );
}

function BulletList({
  blockId,
  bullets,
  editable,
  labelPrefix,
  onChange,
}: {
  blockId: string;
  bullets: string[];
  editable: boolean;
  labelPrefix: string;
  onChange: (item: number, value: string) => void;
}) {
  if (bullets.length === 0) return null;
  return (
    <ul className="m-0 mt-[calc(2px*var(--resume-sp))] flex list-none flex-col gap-[calc(2.5px*var(--resume-sp))] p-0">
      {bullets.map((bullet, item) => (
        <li
          key={`${blockId}-bullet-${item}`}
          className="relative pl-[11px] text-[calc(11.3px*var(--resume-fs))] leading-[1.45] text-[var(--resume-body)] before:absolute before:top-[0.55em] before:left-px before:h-[3px] before:w-[3px] before:rounded-full before:bg-[var(--resume-accent-soft)] before:content-['']"
        >
          <ResumeEditableText
            tag="span"
            editable={editable}
            multiline
            label={`${labelPrefix}.${item + 1}`}
            value={bullet}
            onChange={(value) => onChange(item, value)}
          />
        </li>
      ))}
    </ul>
  );
}

function SkillsSection({
  groups,
  editable,
  onChange,
}: {
  groups: ResumeSkillGroup[];
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  // Un groupe vidé de tous ses items par `removeSkill` ne doit pas laisser un intitulé de
  // groupe orphelin, sans puce — l'index d'origine reste utilisé pour cibler `onChange`,
  // le filtrage ne sert qu'à décider quoi afficher.
  const groupesNonVides = groups
    .map((group, groupIndex) => ({ group, groupIndex }))
    .filter(({ group }) => group.items.length > 0);
  if (groupesNonVides.length === 0) return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Compétences</h2>
      </div>
      <div className="flex flex-col gap-[calc(5px*var(--resume-sp))]">
        {groupesNonVides.map(({ group, groupIndex }) => (
          <div key={group.id} className="grid grid-cols-[27%_1fr] items-baseline gap-[14px]">
            <h3 className="m-0 text-[calc(11px*var(--resume-fs))] font-semibold tracking-[-0.004em] text-[var(--resume-ink)]">
              {group.name}
            </h3>
            <ul className="m-0 flex list-none flex-wrap gap-x-[5px] gap-y-[calc(3.5px*var(--resume-sp))] p-0">
              {group.items.map((item, itemIndex) => (
                <li
                  key={`${group.id}-item-${itemIndex}`}
                  className="rounded bg-[var(--resume-chip)] px-2 pt-[2.5px] pb-[3px] text-[calc(10.4px*var(--resume-fs))] leading-[1.32] text-[var(--resume-ink)]"
                >
                  <ResumeEditableText
                    tag="span"
                    editable={editable}
                    label={`Compétence ${itemIndex + 1} du groupe ${group.name || groupIndex + 1}`}
                    value={item}
                    onChange={(value) => onChange({ type: "skill", group: groupIndex, item: itemIndex }, value)}
                  />
                </li>
              ))}
            </ul>
          </div>
        ))}
      </div>
    </section>
  );
}

function EducationSection({
  education,
  editable,
  onChange,
}: {
  education: ResumeEducationBlock[];
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  if (education.length === 0) return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Formation</h2>
      </div>
      <div className="flex flex-col gap-[calc(6px*var(--resume-sp))]">
        {education.map((entry, index) => (
          <div key={entry.id} className="flex flex-col gap-[calc(1.5px*var(--resume-sp))]">
            <div className="flex items-baseline justify-between gap-[14px]">
              <ResumeEditableText
                tag="h3"
                editable={editable}
                label={`Diplôme ${index + 1}`}
                value={entry.degree}
                className="m-0 text-[calc(12px*var(--resume-fs))] font-semibold text-[var(--resume-ink)]"
                onChange={(value) => onChange({ type: "education", index, field: "degree" }, value)}
              />
              <ResumeEditableText
                tag="span"
                editable={editable}
                label={`Période de formation ${index + 1}`}
                value={entry.period}
                className={META}
                onChange={(value) => onChange({ type: "education", index, field: "period" }, value)}
              />
            </div>
            <p className="m-0 text-[calc(11.2px*var(--resume-fs))] leading-[1.4] text-[var(--resume-muted)]">
              <ResumeEditableText
                tag="span"
                editable={editable}
                label={`École ${index + 1}`}
                value={entry.school}
                onChange={(value) => onChange({ type: "education", index, field: "school" }, value)}
              />
              {entry.location ? (
                <>
                  <span className={SEPARATOR}>&nbsp;·&nbsp;</span>
                  <ResumeEditableText
                    tag="span"
                    editable={editable}
                    label={`Lieu de formation ${index + 1}`}
                    value={entry.location}
                    onChange={(value) => onChange({ type: "education", index, field: "location" }, value)}
                  />
                </>
              ) : null}
            </p>
            {entry.description ? (
              <ResumeEditableText
                tag="p"
                editable={editable}
                multiline
                label={`Mention ${index + 1}`}
                value={entry.description}
                className="m-0 text-[calc(10.9px*var(--resume-fs))] leading-[1.4] text-[var(--resume-subtle)]"
                onChange={(value) => onChange({ type: "education", index, field: "description" }, value)}
              />
            ) : null}
          </div>
        ))}
      </div>
    </section>
  );
}

function CertificationsSection({
  certifications,
  editable,
  onChange,
}: {
  certifications: ResumeCertificationBlock[];
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  if (certifications.length === 0) return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Certifications</h2>
      </div>
      <ul className="m-0 flex list-none flex-col gap-[calc(2.5px*var(--resume-sp))] p-0">
        {certifications.map((certification, index) => (
          <li key={certification.id} className="text-[calc(11.2px*var(--resume-fs))] leading-[1.42] text-[var(--resume-body)]">
            <ResumeEditableText
              tag="span"
              editable={editable}
              label={`Certification ${index + 1}`}
              value={certification.name}
              onChange={(value) => onChange({ type: "certification", index, field: "name" }, value)}
            />
            {certification.issuer ? (
              <>
                {" · "}
                <ResumeEditableText
                  tag="span"
                  editable={editable}
                  label={`Organisme de certification ${index + 1}`}
                  value={certification.issuer}
                  onChange={(value) => onChange({ type: "certification", index, field: "issuer" }, value)}
                />
              </>
            ) : null}
            {certification.date ? (
              <>
                {" · "}
                <ResumeEditableText
                  tag="span"
                  editable={editable}
                  label={`Année de certification ${index + 1}`}
                  value={certification.date}
                  onChange={(value) => onChange({ type: "certification", index, field: "date" }, value)}
                />
              </>
            ) : null}
          </li>
        ))}
      </ul>
    </section>
  );
}

function LanguagesSection({
  languages,
  editable,
  onChange,
}: {
  languages: ResumeLanguageBlock[];
  editable: boolean;
  onChange: ResumeFieldChange;
}) {
  if (languages.length === 0) return null;
  return (
    <section className={GRID}>
      <div className="pt-[calc(2px*var(--resume-fs))]">
        <h2 className={LABEL}>Langues</h2>
      </div>
      <ul className="m-0 flex list-none flex-wrap gap-x-[22px] gap-y-[calc(2px*var(--resume-sp))] p-0">
        {languages.map((language, index) => (
          <li key={language.id} className="text-[calc(11.2px*var(--resume-fs))] leading-[1.42] text-[var(--resume-body)]">
            <ResumeEditableText
              tag="span"
              editable={editable}
              label={`Langue ${index + 1}`}
              value={language.name}
              onChange={(value) => onChange({ type: "language", index, field: "name" }, value)}
            />
            {" · "}
            <ResumeEditableText
              tag="span"
              editable={editable}
              label={`Niveau ${index + 1}`}
              value={language.level}
              onChange={(value) => onChange({ type: "language", index, field: "level" }, value)}
            />
          </li>
        ))}
      </ul>
    </section>
  );
}
