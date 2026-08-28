import { useState } from "react";
import type {
  Certification,
  Skill,
  Experience,
  Education,
  Identity,
  Language,
  Profile,
  Project,
} from "@/shared/types/generated/profile";
import { Button, FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";
import {
  certificationsSchema,
  skillsSchema,
  experiencesSchema,
  educationSchema,
  identitySchema,
  languagesSchema,
  projectsSchema,
} from "../../model/profileSchemas";

export type ProfileSection =
  | "identity"
  | "experiences"
  | "skills"
  | "education"
  | "languages"
  | "projects"
  | "certifications";

const META: Record<ProfileSection, { icon: string; title: string; subtitle: string }> = {
  identity: { icon: "person", title: "Identité et objectif", subtitle: "Présentez votre projet professionnel" },
  experiences: { icon: "work_history", title: "Expériences", subtitle: "Décrivez les étapes utiles de votre parcours" },
  skills: { icon: "psychology", title: "Compétences", subtitle: "Ajoutez vos savoir-faire principaux" },
  education: { icon: "school", title: "Formations", subtitle: "Diplômes et parcours de formation" },
  languages: { icon: "translate", title: "Langues", subtitle: "Indiquez votre niveau de pratique" },
  projects: { icon: "rocket_launch", title: "Projets", subtitle: "Valorisez vos réalisations personnelles" },
  certifications: { icon: "workspace_premium", title: "Certifications", subtitle: "Ajoutez vos qualifications reconnues" },
};

type Errors = Record<string, string>;

const experienceVide = (): Experience => ({
  title: "", company: "", location: null, start_date: "", end_date: null, current: false, description: null,
});
const educationVide = (): Education => ({
  degree: "", school: "", location: null, start_date: null, end_date: null, description: null,
});
const languageVide = (): Language => ({ name: "", level: "" });
const projectVide = (): Project => ({ name: "", description: null, url: null, technologies: null });
const certificationVide = (): Certification => ({ name: "", issuer: null, date: null, url: null });
const text = (value: string | null): string => value ?? "";

/** Éditeur commun aux sept sous-sections persistées du profil. */
export function ProfileSectionModal({
  section,
  profile,
  busy,
  onClose,
  onSubmit,
}: {
  section: ProfileSection;
  profile: Profile;
  busy: boolean;
  onClose: () => void;
  onSubmit: (profile: Profile) => Promise<unknown>;
}) {
  const [draft, setDraft] = useState<Profile>(() => structuredClone(profile));
  const [errors, setErrors] = useState<Errors>({});
  const [newSkill, setNewSkill] = useState("");

  const meta = META[section];

  const save = async () => {
    const resultat = valider(section, draft);
    if (!resultat.success) {
      setErrors(resultat.errors);
      return;
    }
    setErrors({});
    await onSubmit(resultat.profile);
    onClose();
  };

  const addSkill = () => {
    const name = newSkill.trim();
    if (!name || draft.skills.some((item) => item.name.toLocaleLowerCase() === name.toLocaleLowerCase())) return;
    setDraft((current) => ({ ...current, skills: [...current.skills, { name }] }));
    setNewSkill("");
  };

  return (
    <ModalHost
      open
      icon={meta.icon}
      title={meta.title}
      subtitle={meta.subtitle}
      footer_note="Les informations sont utilisées dans votre CV."
      busy={busy}
      onClose={onClose}
      onSubmit={() => void save()}
      width={section === "identity" ? "720px" : "760px"}
    >
      <form
        onSubmit={(event) => { event.preventDefault(); void save(); }}
        className="flex flex-col gap-4"
      >
        {section === "identity" ? (
          <IdentityForm
            value={draft.identity}
            errors={errors}
            onChange={(identity) => setDraft((current) => ({ ...current, identity }))}
          />
        ) : null}
        {section === "experiences" ? (
          <ExperiencesForm
            value={draft.experiences}
            errors={errors}
            onChange={(experiences) => setDraft((current) => ({ ...current, experiences }))}
          />
        ) : null}
        {section === "skills" ? (
          <SkillsForm
            value={draft.skills}
            input={newSkill}
            error={errors["0.name"]}
            onInput={setNewSkill}
            onAdd={addSkill}
            onChange={(skills) => setDraft((current) => ({ ...current, skills }))}
          />
        ) : null}
        {section === "education" ? (
          <EducationForm value={draft.education} errors={errors} onChange={(education) => setDraft((current) => ({ ...current, education }))} />
        ) : null}
        {section === "languages" ? (
          <LanguagesForm value={draft.languages} errors={errors} onChange={(languages) => setDraft((current) => ({ ...current, languages }))} />
        ) : null}
        {section === "projects" ? (
          <ProjectsForm value={draft.projects} errors={errors} onChange={(projects) => setDraft((current) => ({ ...current, projects }))} />
        ) : null}
        {section === "certifications" ? (
          <CertificationsForm value={draft.certifications} errors={errors} onChange={(certifications) => setDraft((current) => ({ ...current, certifications }))} />
        ) : null}
      </form>
    </ModalHost>
  );
}

function IdentityForm({ value, errors, onChange }: { value: Identity; errors: Errors; onChange: (value: Identity) => void }) {
  const change = (field: keyof Identity, next: string) => onChange({ ...value, [field]: next });
  return (
    <>
      <fieldset className="grid gap-4 sm:grid-cols-2">
        <legend className="sr-only">Coordonnées</legend>
        <Champ label="Prénom" value={value.first_name} error={errors.first_name} onChange={(v) => change("first_name", v)} />
        <Champ label="Nom" value={value.name} error={errors.name} onChange={(v) => change("name", v)} />
        <Champ label="E-mail" type="email" value={value.email} error={errors.email} onChange={(v) => change("email", v)} />
        <Champ label="Téléphone" type="tel" value={text(value.phone)} error={errors.phone} onChange={(v) => change("phone", v)} />
        <div className="sm:col-span-2"><Champ label="Ville" value={text(value.city)} error={errors.city} onChange={(v) => change("city", v)} /></div>
      </fieldset>
      <fieldset className="grid gap-4 border-t border-line pt-4">
        <legend className="mb-3 text-eyebrow uppercase text-ink-faint">Objectif professionnel</legend>
        <Champ label="Titre ou poste visé" value={text(value.title)} error={errors.title} placeholder="Product designer — mobilité durable" onChange={(v) => change("title", v)} />
        <FormField label="Présentation" error={errors.resume} help="En quelques phrases : votre expérience, vos forces et ce que vous recherchez.">
          {(props) => <TextArea {...props} rows={5} value={text(value.resume)} invalid={Boolean(errors.resume)} onChange={(event) => change("resume", event.target.value)} />}
        </FormField>
      </fieldset>
      <fieldset className="grid gap-4 border-t border-line pt-4 sm:grid-cols-2">
        <legend className="mb-3 text-eyebrow uppercase text-ink-faint sm:col-span-2">Présence en ligne</legend>
        <Champ label="LinkedIn" type="url" value={text(value.linkedin)} error={errors.linkedin} placeholder="https://linkedin.com/in/…" onChange={(v) => change("linkedin", v)} />
        <Champ label="GitHub" type="url" value={text(value.github)} error={errors.github} placeholder="https://github.com/…" onChange={(v) => change("github", v)} />
        <div className="sm:col-span-2"><Champ label="Site web" type="url" value={text(value.website)} error={errors.website} placeholder="https://…" onChange={(v) => change("website", v)} /></div>
      </fieldset>
    </>
  );
}

function ExperiencesForm({ value, errors, onChange }: { value: Experience[]; errors: Errors; onChange: (value: Experience[]) => void }) {
  const update = (index: number, patch: Partial<Experience>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return (
    <RepeatList empty="Aucune expérience ajoutée" addLabel="Ajouter une expérience" onAdd={() => onChange([...value, experienceVide()])}>
      {value.map((item, index) => (
        <ItemCard key={index} title={item.title || `Expérience ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}>
          <div className="grid gap-4 sm:grid-cols-2">
            <Champ required label="Intitulé" value={item.title} error={errors[`${index}.title`]} onChange={(v) => update(index, { title: v })} />
            <Champ required label="Entreprise" value={item.company} error={errors[`${index}.company`]} onChange={(v) => update(index, { company: v })} />
            <Champ label="Lieu" value={text(item.location)} error={errors[`${index}.location`]} onChange={(v) => update(index, { location: v })} />
            <Champ required label="Début" value={item.start_date} error={errors[`${index}.start_date`]} placeholder="AAAA-MM" onChange={(v) => update(index, { start_date: v })} />
            <Champ label="Fin" value={text(item.end_date)} disabled={item.current} error={errors[`${index}.end_date`]} placeholder="AAAA-MM" onChange={(v) => update(index, { end_date: v })} />
            <label className="flex min-h-field items-center gap-2 self-end text-body text-ink-muted"><input type="checkbox" checked={item.current} onChange={(event) => update(index, { current: event.target.checked, ...(event.target.checked ? { end_date: null } : {}) })} /> Poste actuel</label>
            <div className="sm:col-span-2"><Zone label="Description" value={text(item.description)} error={errors[`${index}.description`]} onChange={(v) => update(index, { description: v })} /></div>
          </div>
        </ItemCard>
      ))}
    </RepeatList>
  );
}

function SkillsForm({ value, input, error, onInput, onAdd, onChange }: { value: Skill[]; input: string; error?: string | undefined; onInput: (value: string) => void; onAdd: () => void; onChange: (value: Skill[]) => void }) {
  return (
    <div className="space-y-4">
      <FormField label="Nouvelle compétence" error={error} help="Entrée permet aussi d'ajouter la compétence.">
        {(props) => <div className="flex gap-2"><TextInput {...props} value={input} placeholder="Ex. Figma, Rust, Gestion de projet" onChange={(event) => onInput(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") { event.preventDefault(); onAdd(); } }} /><Button icon="add" onClick={onAdd}>Add</Button></div>}
      </FormField>
      {value.length === 0 ? <EmptyInline text="Aucune compétence ajoutée" /> : (
        <ul aria-label="Compétences ajoutées" className="flex flex-wrap gap-2">
          {value.map((item, index) => <li key={`${item.name}-${index}`} className="flex items-center gap-1 rounded-full border border-accent-border bg-accent-tint px-3 py-1.5 text-body font-medium text-accent">{item.name}<button type="button" aria-label={`Retirer ${item.name}`} onClick={() => onChange(value.filter((_, i) => i !== index))} className="ml-1 rounded-full px-1 hover:bg-accent/10">×</button></li>)}
        </ul>
      )}
    </div>
  );
}

function EducationForm({ value, errors, onChange }: { value: Education[]; errors: Errors; onChange: (value: Education[]) => void }) {
  const update = (index: number, patch: Partial<Education>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucune formation ajoutée" addLabel="Ajouter une formation" onAdd={() => onChange([...value, educationVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.degree || `Formation ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4 sm:grid-cols-2"><Champ required label="Diplôme" value={item.degree} error={errors[`${index}.degree`]} onChange={(v) => update(index, { degree: v })} /><Champ required label="Établissement" value={item.school} error={errors[`${index}.school`]} onChange={(v) => update(index, { school: v })} /><Champ label="Lieu" value={text(item.location)} onChange={(v) => update(index, { location: v })} /><div className="grid grid-cols-2 gap-3"><Champ label="Début" value={text(item.start_date)} placeholder="AAAA" onChange={(v) => update(index, { start_date: v })} /><Champ label="Fin" value={text(item.end_date)} placeholder="AAAA" onChange={(v) => update(index, { end_date: v })} /></div><div className="sm:col-span-2"><Zone label="Description" value={text(item.description)} onChange={(v) => update(index, { description: v })} /></div></div></ItemCard>)}</RepeatList>;
}

function LanguagesForm({ value, errors, onChange }: { value: Language[]; errors: Errors; onChange: (value: Language[]) => void }) {
  const update = (index: number, patch: Partial<Language>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucune langue ajoutée" addLabel="Ajouter une langue" onAdd={() => onChange([...value, languageVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.name || `Langue ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4 sm:grid-cols-2"><Champ required label="Langue" value={item.name} error={errors[`${index}.name`]} onChange={(v) => update(index, { name: v })} /><FormField label="Niveau" required error={errors[`${index}.level`]}>{(props) => <Select {...props} value={item.level} invalid={Boolean(errors[`${index}.level`])} onChange={(event) => update(index, { level: event.target.value })}><option value="">Choisir…</option><option>Débutant</option><option>Intermédiaire</option><option>Professionnel</option><option>Courant</option><option>Langue maternelle</option></Select>}</FormField></div></ItemCard>)}</RepeatList>;
}

function ProjectsForm({ value, errors, onChange }: { value: Project[]; errors: Errors; onChange: (value: Project[]) => void }) {
  const update = (index: number, patch: Partial<Project>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucun projet ajouté" addLabel="Ajouter un projet" onAdd={() => onChange([...value, projectVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.name || `Projet ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4"><Champ required label="Nom" value={item.name} error={errors[`${index}.name`]} onChange={(v) => update(index, { name: v })} /><Zone label="Description" value={text(item.description)} onChange={(v) => update(index, { description: v })} /><div className="grid gap-4 sm:grid-cols-2"><Champ label="Lien" type="url" value={text(item.url)} error={errors[`${index}.url`]} onChange={(v) => update(index, { url: v })} /><Champ label="Technologies" value={text(item.technologies)} onChange={(v) => update(index, { technologies: v })} /></div></div></ItemCard>)}</RepeatList>;
}

function CertificationsForm({ value, errors, onChange }: { value: Certification[]; errors: Errors; onChange: (value: Certification[]) => void }) {
  const update = (index: number, patch: Partial<Certification>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucune certification ajoutée" addLabel="Ajouter une certification" onAdd={() => onChange([...value, certificationVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.name || `Certification ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4 sm:grid-cols-2"><Champ required label="Nom" value={item.name} error={errors[`${index}.name`]} onChange={(v) => update(index, { name: v })} /><Champ label="Organisme" value={text(item.issuer)} onChange={(v) => update(index, { issuer: v })} /><Champ label="Date" value={text(item.date)} placeholder="AAAA-MM" onChange={(v) => update(index, { date: v })} /><Champ label="Lien" type="url" value={text(item.url)} error={errors[`${index}.url`]} onChange={(v) => update(index, { url: v })} /></div></ItemCard>)}</RepeatList>;
}

function Champ({ label, value, onChange, error, required = false, type = "text", placeholder, disabled = false }: { label: string; value: string; onChange: (value: string) => void; error?: string | undefined; required?: boolean | undefined; type?: string | undefined; placeholder?: string | undefined; disabled?: boolean | undefined }) {
  return <FormField label={label} required={required} error={error}>{(props) => <TextInput {...props} type={type} value={value} placeholder={placeholder} disabled={disabled} invalid={Boolean(error)} onChange={(event) => onChange(event.target.value)} />}</FormField>;
}

function Zone({ label, value, onChange, error }: { label: string; value: string; onChange: (value: string) => void; error?: string | undefined }) {
  return <FormField label={label} error={error}>{(props) => <TextArea {...props} value={value} invalid={Boolean(error)} onChange={(event) => onChange(event.target.value)} />}</FormField>;
}

function RepeatList({ empty, addLabel, onAdd, children }: { empty: string; addLabel: string; onAdd: () => void; children: React.ReactNode }) {
  const aucun = Array.isArray(children) && children.length === 0;
  return <div className="space-y-4">{aucun ? <EmptyInline text={empty} /> : children}<Button variant="secondary" icon="add" onClick={onAdd}>{addLabel}</Button></div>;
}

function ItemCard({ title, onRemove, children }: { title: string; onRemove: () => void; children: React.ReactNode }) {
  return <fieldset className="rounded-card border border-line bg-surface-alt p-4"><legend className="sr-only">{title}</legend><div className="mb-4 flex items-center gap-2"><p className="min-w-0 flex-1 truncate text-section text-ink">{title}</p><Button variant="ghost" icon="delete" aria-label={`Supprimer ${title}`} onClick={onRemove}>Supprimer</Button></div>{children}</fieldset>;
}

function EmptyInline({ text }: { text: string }) {
  return <p className="rounded-card border border-dashed border-line bg-surface-alt px-4 py-8 text-center text-body text-ink-muted">{text}</p>;
}

type Validation = { success: true; profile: Profile } | { success: false; errors: Errors };

function valider(section: ProfileSection, profile: Profile): Validation {
  const schemaEtValue = (() => {
    switch (section) {
      case "identity": return [identitySchema, profile.identity] as const;
      case "experiences": return [experiencesSchema, profile.experiences] as const;
      case "skills": return [skillsSchema, profile.skills] as const;
      case "education": return [educationSchema, profile.education] as const;
      case "languages": return [languagesSchema, profile.languages] as const;
      case "projects": return [projectsSchema, profile.projects] as const;
      case "certifications": return [certificationsSchema, profile.certifications] as const;
    }
  })();
  const resultat = schemaEtValue[0].safeParse(schemaEtValue[1]);
  if (!resultat.success) {
    return { success: false, errors: Object.fromEntries(resultat.error.issues.map((issue) => [issue.path.join("."), issue.message])) };
  }
  return { success: true, profile: { ...profile, [section]: resultat.data } };
}
