import { useState } from "react";
import type {
  Certification,
  Competence,
  Experience,
  Formation,
  Identite,
  Langue,
  Profil,
  Projet,
} from "@/shared/types/generated/profil";
import { Button, FormField, ModalHost, Select, TextArea, TextInput } from "@/shared/ui";
import {
  certificationsSchema,
  competencesSchema,
  experiencesSchema,
  formationsSchema,
  identiteSchema,
  languesSchema,
  projetsSchema,
} from "../../model/profil.schemas";

export type ProfilSection =
  | "identite"
  | "experiences"
  | "competences"
  | "formations"
  | "langues"
  | "projets"
  | "certifications";

const META: Record<ProfilSection, { icon: string; title: string; subtitle: string }> = {
  identite: { icon: "person", title: "Identité et objectif", subtitle: "Présentez votre projet professionnel" },
  experiences: { icon: "work_history", title: "Expériences", subtitle: "Décrivez les étapes utiles de votre parcours" },
  competences: { icon: "psychology", title: "Compétences", subtitle: "Ajoutez vos savoir-faire principaux" },
  formations: { icon: "school", title: "Formations", subtitle: "Diplômes et parcours de formation" },
  langues: { icon: "translate", title: "Langues", subtitle: "Indiquez votre niveau de pratique" },
  projets: { icon: "rocket_launch", title: "Projets", subtitle: "Valorisez vos réalisations personnelles" },
  certifications: { icon: "workspace_premium", title: "Certifications", subtitle: "Ajoutez vos qualifications reconnues" },
};

type Errors = Record<string, string>;

const experienceVide = (): Experience => ({
  intitule: "", entreprise: "", lieu: null, dateDebut: "", dateFin: null, posteActuel: false, description: null,
});
const formationVide = (): Formation => ({
  diplome: "", etablissement: "", lieu: null, dateDebut: null, dateFin: null, description: null,
});
const langueVide = (): Langue => ({ nom: "", niveau: "" });
const projetVide = (): Projet => ({ nom: "", description: null, url: null, technologies: null });
const certificationVide = (): Certification => ({ nom: "", organisme: null, date: null, url: null });
const texte = (valeur: string | null): string => valeur ?? "";

/** Éditeur commun aux sept sous-sections persistées du profil. */
export function ProfilSectionModal({
  section,
  profil,
  busy,
  onClose,
  onSubmit,
}: {
  section: ProfilSection;
  profil: Profil;
  busy: boolean;
  onClose: () => void;
  onSubmit: (profil: Profil) => Promise<unknown>;
}) {
  const [draft, setDraft] = useState<Profil>(() => structuredClone(profil));
  const [errors, setErrors] = useState<Errors>({});
  const [nouvelleCompetence, setNouvelleCompetence] = useState("");

  const meta = META[section];

  const enregistrer = async () => {
    const resultat = valider(section, draft);
    if (!resultat.success) {
      setErrors(resultat.errors);
      return;
    }
    setErrors({});
    await onSubmit(resultat.profil);
    onClose();
  };

  const ajouterCompetence = () => {
    const nom = nouvelleCompetence.trim();
    if (!nom || draft.competences.some((item) => item.nom.toLocaleLowerCase() === nom.toLocaleLowerCase())) return;
    setDraft((actuel) => ({ ...actuel, competences: [...actuel.competences, { nom }] }));
    setNouvelleCompetence("");
  };

  return (
    <ModalHost
      open
      icon={meta.icon}
      title={meta.title}
      subtitle={meta.subtitle}
      footerNote="Les informations sont utilisées dans votre CV."
      busy={busy}
      onClose={onClose}
      onSubmit={() => void enregistrer()}
      width={section === "identite" ? "720px" : "760px"}
    >
      <form
        onSubmit={(event) => { event.preventDefault(); void enregistrer(); }}
        className="flex flex-col gap-4"
      >
        {section === "identite" ? (
          <IdentiteForm
            value={draft.identite}
            errors={errors}
            onChange={(identite) => setDraft((actuel) => ({ ...actuel, identite }))}
          />
        ) : null}
        {section === "experiences" ? (
          <ExperiencesForm
            value={draft.experiences}
            errors={errors}
            onChange={(experiences) => setDraft((actuel) => ({ ...actuel, experiences }))}
          />
        ) : null}
        {section === "competences" ? (
          <CompetencesForm
            value={draft.competences}
            input={nouvelleCompetence}
            error={errors["0.nom"]}
            onInput={setNouvelleCompetence}
            onAdd={ajouterCompetence}
            onChange={(competences) => setDraft((actuel) => ({ ...actuel, competences }))}
          />
        ) : null}
        {section === "formations" ? (
          <FormationsForm value={draft.formations} errors={errors} onChange={(formations) => setDraft((actuel) => ({ ...actuel, formations }))} />
        ) : null}
        {section === "langues" ? (
          <LanguesForm value={draft.langues} errors={errors} onChange={(langues) => setDraft((actuel) => ({ ...actuel, langues }))} />
        ) : null}
        {section === "projets" ? (
          <ProjetsForm value={draft.projets} errors={errors} onChange={(projets) => setDraft((actuel) => ({ ...actuel, projets }))} />
        ) : null}
        {section === "certifications" ? (
          <CertificationsForm value={draft.certifications} errors={errors} onChange={(certifications) => setDraft((actuel) => ({ ...actuel, certifications }))} />
        ) : null}
      </form>
    </ModalHost>
  );
}

function IdentiteForm({ value, errors, onChange }: { value: Identite; errors: Errors; onChange: (value: Identite) => void }) {
  const change = (champ: keyof Identite, valeur: string) => onChange({ ...value, [champ]: valeur });
  return (
    <>
      <fieldset className="grid gap-4 sm:grid-cols-2">
        <legend className="sr-only">Coordonnées</legend>
        <Champ label="Prénom" value={value.prenom} error={errors.prenom} onChange={(v) => change("prenom", v)} />
        <Champ label="Nom" value={value.nom} error={errors.nom} onChange={(v) => change("nom", v)} />
        <Champ label="E-mail" type="email" value={value.email} error={errors.email} onChange={(v) => change("email", v)} />
        <Champ label="Téléphone" type="tel" value={texte(value.telephone)} error={errors.telephone} onChange={(v) => change("telephone", v)} />
        <div className="sm:col-span-2"><Champ label="Ville" value={texte(value.ville)} error={errors.ville} onChange={(v) => change("ville", v)} /></div>
      </fieldset>
      <fieldset className="grid gap-4 border-t border-line pt-4">
        <legend className="mb-3 text-eyebrow uppercase text-ink-faint">Objectif professionnel</legend>
        <Champ label="Titre ou poste visé" value={texte(value.titre)} error={errors.titre} placeholder="Product designer — mobilité durable" onChange={(v) => change("titre", v)} />
        <FormField label="Présentation" error={errors.resume} help="En quelques phrases : votre expérience, vos forces et ce que vous recherchez.">
          {(props) => <TextArea {...props} rows={5} value={texte(value.resume)} invalid={Boolean(errors.resume)} onChange={(event) => change("resume", event.target.value)} />}
        </FormField>
      </fieldset>
      <fieldset className="grid gap-4 border-t border-line pt-4 sm:grid-cols-2">
        <legend className="mb-3 text-eyebrow uppercase text-ink-faint sm:col-span-2">Présence en ligne</legend>
        <Champ label="LinkedIn" type="url" value={texte(value.linkedin)} error={errors.linkedin} placeholder="https://linkedin.com/in/…" onChange={(v) => change("linkedin", v)} />
        <Champ label="GitHub" type="url" value={texte(value.github)} error={errors.github} placeholder="https://github.com/…" onChange={(v) => change("github", v)} />
        <div className="sm:col-span-2"><Champ label="Site web" type="url" value={texte(value.siteWeb)} error={errors.siteWeb} placeholder="https://…" onChange={(v) => change("siteWeb", v)} /></div>
      </fieldset>
    </>
  );
}

function ExperiencesForm({ value, errors, onChange }: { value: Experience[]; errors: Errors; onChange: (value: Experience[]) => void }) {
  const update = (index: number, patch: Partial<Experience>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return (
    <RepeatList empty="Aucune expérience ajoutée" addLabel="Ajouter une expérience" onAdd={() => onChange([...value, experienceVide()])}>
      {value.map((item, index) => (
        <ItemCard key={index} title={item.intitule || `Expérience ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}>
          <div className="grid gap-4 sm:grid-cols-2">
            <Champ required label="Intitulé" value={item.intitule} error={errors[`${index}.intitule`]} onChange={(v) => update(index, { intitule: v })} />
            <Champ required label="Entreprise" value={item.entreprise} error={errors[`${index}.entreprise`]} onChange={(v) => update(index, { entreprise: v })} />
            <Champ label="Lieu" value={texte(item.lieu)} error={errors[`${index}.lieu`]} onChange={(v) => update(index, { lieu: v })} />
            <Champ required label="Début" value={item.dateDebut} error={errors[`${index}.dateDebut`]} placeholder="AAAA-MM" onChange={(v) => update(index, { dateDebut: v })} />
            <Champ label="Fin" value={texte(item.dateFin)} disabled={item.posteActuel} error={errors[`${index}.dateFin`]} placeholder="AAAA-MM" onChange={(v) => update(index, { dateFin: v })} />
            <label className="flex min-h-field items-center gap-2 self-end text-body text-ink-muted"><input type="checkbox" checked={item.posteActuel} onChange={(event) => update(index, { posteActuel: event.target.checked, ...(event.target.checked ? { dateFin: null } : {}) })} /> Poste actuel</label>
            <div className="sm:col-span-2"><Zone label="Description" value={texte(item.description)} error={errors[`${index}.description`]} onChange={(v) => update(index, { description: v })} /></div>
          </div>
        </ItemCard>
      ))}
    </RepeatList>
  );
}

function CompetencesForm({ value, input, error, onInput, onAdd, onChange }: { value: Competence[]; input: string; error?: string | undefined; onInput: (value: string) => void; onAdd: () => void; onChange: (value: Competence[]) => void }) {
  return (
    <div className="space-y-4">
      <FormField label="Nouvelle compétence" error={error} help="Entrée permet aussi d'ajouter la compétence.">
        {(props) => <div className="flex gap-2"><TextInput {...props} value={input} placeholder="Ex. Figma, Rust, Gestion de projet" onChange={(event) => onInput(event.target.value)} onKeyDown={(event) => { if (event.key === "Enter") { event.preventDefault(); onAdd(); } }} /><Button icon="add" onClick={onAdd}>Ajouter</Button></div>}
      </FormField>
      {value.length === 0 ? <EmptyInline text="Aucune compétence ajoutée" /> : (
        <ul aria-label="Compétences ajoutées" className="flex flex-wrap gap-2">
          {value.map((item, index) => <li key={`${item.nom}-${index}`} className="flex items-center gap-1 rounded-full border border-accent-border bg-accent-tint px-3 py-1.5 text-body font-medium text-accent">{item.nom}<button type="button" aria-label={`Retirer ${item.nom}`} onClick={() => onChange(value.filter((_, i) => i !== index))} className="ml-1 rounded-full px-1 hover:bg-accent/10">×</button></li>)}
        </ul>
      )}
    </div>
  );
}

function FormationsForm({ value, errors, onChange }: { value: Formation[]; errors: Errors; onChange: (value: Formation[]) => void }) {
  const update = (index: number, patch: Partial<Formation>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucune formation ajoutée" addLabel="Ajouter une formation" onAdd={() => onChange([...value, formationVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.diplome || `Formation ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4 sm:grid-cols-2"><Champ required label="Diplôme" value={item.diplome} error={errors[`${index}.diplome`]} onChange={(v) => update(index, { diplome: v })} /><Champ required label="Établissement" value={item.etablissement} error={errors[`${index}.etablissement`]} onChange={(v) => update(index, { etablissement: v })} /><Champ label="Lieu" value={texte(item.lieu)} onChange={(v) => update(index, { lieu: v })} /><div className="grid grid-cols-2 gap-3"><Champ label="Début" value={texte(item.dateDebut)} placeholder="AAAA" onChange={(v) => update(index, { dateDebut: v })} /><Champ label="Fin" value={texte(item.dateFin)} placeholder="AAAA" onChange={(v) => update(index, { dateFin: v })} /></div><div className="sm:col-span-2"><Zone label="Description" value={texte(item.description)} onChange={(v) => update(index, { description: v })} /></div></div></ItemCard>)}</RepeatList>;
}

function LanguesForm({ value, errors, onChange }: { value: Langue[]; errors: Errors; onChange: (value: Langue[]) => void }) {
  const update = (index: number, patch: Partial<Langue>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucune langue ajoutée" addLabel="Ajouter une langue" onAdd={() => onChange([...value, langueVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.nom || `Langue ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4 sm:grid-cols-2"><Champ required label="Langue" value={item.nom} error={errors[`${index}.nom`]} onChange={(v) => update(index, { nom: v })} /><FormField label="Niveau" required error={errors[`${index}.niveau`]}>{(props) => <Select {...props} value={item.niveau} invalid={Boolean(errors[`${index}.niveau`])} onChange={(event) => update(index, { niveau: event.target.value })}><option value="">Choisir…</option><option>Débutant</option><option>Intermédiaire</option><option>Professionnel</option><option>Courant</option><option>Langue maternelle</option></Select>}</FormField></div></ItemCard>)}</RepeatList>;
}

function ProjetsForm({ value, errors, onChange }: { value: Projet[]; errors: Errors; onChange: (value: Projet[]) => void }) {
  const update = (index: number, patch: Partial<Projet>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucun projet ajouté" addLabel="Ajouter un projet" onAdd={() => onChange([...value, projetVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.nom || `Projet ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4"><Champ required label="Nom" value={item.nom} error={errors[`${index}.nom`]} onChange={(v) => update(index, { nom: v })} /><Zone label="Description" value={texte(item.description)} onChange={(v) => update(index, { description: v })} /><div className="grid gap-4 sm:grid-cols-2"><Champ label="Lien" type="url" value={texte(item.url)} error={errors[`${index}.url`]} onChange={(v) => update(index, { url: v })} /><Champ label="Technologies" value={texte(item.technologies)} onChange={(v) => update(index, { technologies: v })} /></div></div></ItemCard>)}</RepeatList>;
}

function CertificationsForm({ value, errors, onChange }: { value: Certification[]; errors: Errors; onChange: (value: Certification[]) => void }) {
  const update = (index: number, patch: Partial<Certification>) => onChange(value.map((item, i) => i === index ? { ...item, ...patch } : item));
  return <RepeatList empty="Aucune certification ajoutée" addLabel="Ajouter une certification" onAdd={() => onChange([...value, certificationVide()])}>{value.map((item, index) => <ItemCard key={index} title={item.nom || `Certification ${index + 1}`} onRemove={() => onChange(value.filter((_, i) => i !== index))}><div className="grid gap-4 sm:grid-cols-2"><Champ required label="Nom" value={item.nom} error={errors[`${index}.nom`]} onChange={(v) => update(index, { nom: v })} /><Champ label="Organisme" value={texte(item.organisme)} onChange={(v) => update(index, { organisme: v })} /><Champ label="Date" value={texte(item.date)} placeholder="AAAA-MM" onChange={(v) => update(index, { date: v })} /><Champ label="Lien" type="url" value={texte(item.url)} error={errors[`${index}.url`]} onChange={(v) => update(index, { url: v })} /></div></ItemCard>)}</RepeatList>;
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

type Validation = { success: true; profil: Profil } | { success: false; errors: Errors };

function valider(section: ProfilSection, profil: Profil): Validation {
  const schemaEtValeur = (() => {
    switch (section) {
      case "identite": return [identiteSchema, profil.identite] as const;
      case "experiences": return [experiencesSchema, profil.experiences] as const;
      case "competences": return [competencesSchema, profil.competences] as const;
      case "formations": return [formationsSchema, profil.formations] as const;
      case "langues": return [languesSchema, profil.langues] as const;
      case "projets": return [projetsSchema, profil.projets] as const;
      case "certifications": return [certificationsSchema, profil.certifications] as const;
    }
  })();
  const resultat = schemaEtValeur[0].safeParse(schemaEtValeur[1]);
  if (!resultat.success) {
    return { success: false, errors: Object.fromEntries(resultat.error.issues.map((issue) => [issue.path.join("."), issue.message])) };
  }
  return { success: true, profil: { ...profil, [section]: resultat.data } };
}
