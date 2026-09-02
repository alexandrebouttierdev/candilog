import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { AppError } from "@/shared/types/app-error";
import type { Profile } from "@/shared/types/generated/profile";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, Card, CardHeader, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { useProfileViewModel } from "../../viewmodel/useProfileViewModel";
import { ProfileSectionModal, type ProfileSection } from "../components/ProfileSectionModal";
import { ProfileImportModal } from "../components/ProfileImportModal";
import type { IconName } from "@/shared/ui/icon-names";
import {
  CompletionBar,
  ProfileIdentity,
  ProfilePanel,
  ProfileSkeleton,
  ProfileTabs,
  SectionCard,
  type ProfileTab,
} from "../components/ProfileUi";

/** Profile professionnel, objectif et parcours exploités par le générateur de CV. */
export function ProfilePage() {
  const vm = useProfileViewModel();
  const navigate = useNavigate();
  const [tab, setTab] = useState<ProfileTab>("experiences");
  const [section, setSection] = useState<ProfileSection | null>(null);
  const [importOpen, setImportOpen] = useState(false);

  return (
    <div className="flex h-full flex-col">
      <ContextBarAccessory>
        <ContextNote>Source de vérité de vos documents</ContextNote>
      </ContextBarAccessory>
      <PageHeader
        icon="account_circle"
        title="Profil professionnel"
        subtitle="Source de vérité de vos documents"
        secondary={
          <Button variant="secondary" icon="description" onClick={() => void navigate("/documents/cv")}>
            Mes CV
          </Button>
        }
        primary={
          <Button variant="primary" icon="edit" disabled={!vm.data} onClick={() => setSection("identity")}>
            Modifier le profil
          </Button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <ProfileSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="p-6"><ErrorBanner message={vm.error instanceof AppError ? vm.error.message : "Le profil n’a pas pu être chargé."} onRetry={vm.recharger} /></div>
        ) : (
          <div>
            <div className="border-b border-line bg-surface px-7 pt-[22px]">
              <div className="flex flex-wrap items-start gap-4">
                <ProfileIdentity identity={vm.data.profile.identity} />
                <CompletionBar
                  value={vm.data.completion}
                  hint={
                    vm.data.incomplete_sections.length === 0
                      ? "Votre profil contient toutes les sections essentielles."
                      : `Ajoutez ${vm.data.incomplete_sections.slice(0, 2).join(" et ").toLowerCase()} pour atteindre 100 %.`
                  }
                />
              </div>
              <ProfileTabs
                active={tab}
                onChange={setTab}
                counts={{
                  experiences: vm.data.profile.experiences.length,
                  skills: vm.data.profile.skills.length,
                  education: vm.data.profile.education.length,
                  projects: vm.data.profile.projects.length,
                  certifications: vm.data.profile.certifications.length,
                  languages: vm.data.profile.languages.length,
                }}
              />
            </div>

            <div className="flex flex-wrap items-start gap-4 px-7 pt-5 pb-8">
              <div className="flex min-w-0 flex-[1_1_460px] flex-col gap-4">
                <ProfilePanel tab="experiences" active={tab === "experiences"}>
                  <SectionCard icon="work_history" title="Expériences" onEdit={() => setSection("experiences")}>
                    <ExperiencesList profile={vm.data.profile} onEdit={() => setSection("experiences")} />
                  </SectionCard>
                </ProfilePanel>
                <ProfilePanel tab="skills" active={tab === "skills"}>
                  <SectionCard icon="psychology" title="Compétences" actionLabel="Gérer" onEdit={() => setSection("skills")}>
                    <SkillsList profile={vm.data.profile} onEdit={() => setSection("skills")} />
                  </SectionCard>
                </ProfilePanel>
                <ProfilePanel tab="education" active={tab === "education"}>
                  <SectionCard icon="school" title="Formations" onEdit={() => setSection("education")}>
                    <EducationList profile={vm.data.profile} onEdit={() => setSection("education")} />
                  </SectionCard>
                </ProfilePanel>
                <ProfilePanel tab="projects" active={tab === "projects"}>
                  <SectionCard icon="rocket_launch" title="Projets" onEdit={() => setSection("projects")}>
                    <SimpleList items={vm.data.profile.projects.map((item) => ({ title: item.name, meta: item.technologies, body: item.description }))} empty="Aucun projet ajouté" action="Ajouter un projet" onEdit={() => setSection("projects")} />
                  </SectionCard>
                </ProfilePanel>
                <ProfilePanel tab="certifications" active={tab === "certifications"}>
                  <SectionCard icon="workspace_premium" title="Certifications" onEdit={() => setSection("certifications")}>
                    <SimpleList items={vm.data.profile.certifications.map((item) => ({ title: item.name, meta: item.issuer, body: item.date }))} empty="Aucune certification ajoutée" action="Ajouter une certification" onEdit={() => setSection("certifications")} />
                  </SectionCard>
                </ProfilePanel>
                <ProfilePanel tab="languages" active={tab === "languages"}>
                  <SectionCard icon="translate" title="Langues" onEdit={() => setSection("languages")}>
                    <SimpleList items={vm.data.profile.languages.map((item) => ({ title: item.name, meta: item.level, body: null }))} empty="Aucune langue ajoutée" action="Ajouter une langue" onEdit={() => setSection("languages")} />
                  </SectionCard>
                </ProfilePanel>
              </div>

              <div className="flex max-w-[380px] min-w-0 flex-[1_1_300px] flex-col gap-4">
                <Card clipped>
                  <CardHeader
                    compact
                    icon="badge"
                    meta={
                      <button
                        type="button"
                        onClick={() => setSection("identity")}
                        className="inline-flex items-center gap-[5px] text-label font-medium text-accent hover:opacity-80"
                      >
                        Modifier
                      </button>
                    }
                  >
                    Identité
                  </CardHeader>
                  <div className="px-[18px] pt-1 pb-3">
                    <Row label="E-mail" value={vm.data.profile.identity.email} />
                    <Row label="Téléphone" value={vm.data.profile.identity.phone} />
                    <Row label="Adresse" value={vm.data.profile.identity.address} />
                    <Row label="Ville" value={vm.data.profile.identity.city} />
                    <Row label="LinkedIn" value={vm.data.profile.identity.linkedin} />
                    <Row label="GitHub" value={vm.data.profile.identity.github} />
                    <Row label="Site" value={vm.data.profile.identity.website} />
                  </div>
                </Card>

                <div className="rounded-card border border-accent-border bg-accent-tint px-[18px] py-4">
                  <div className="mb-2 flex items-center gap-2">
                    <Icon name="auto_awesome" size={18} className="text-accent" />
                    <span className="text-item font-semibold text-accent">Importer depuis un CV</span>
                  </div>
                  <p className="mb-3 text-label leading-[1.55] text-ink-muted">
                    L'IA extrait vos expériences, compétences et formations depuis un PDF. Vous validez chaque champ avant enregistrement.
                  </p>
                  <Button variant="primary" icon="upload_file" className="w-full" onClick={() => setImportOpen(true)}>
                    Analyser un CV
                  </Button>
                </div>
              </div>
            </div>
          </div>
        )}
      </div>

      {vm.data && section ? <ProfileSectionModal key={section} section={section} profile={vm.data.profile} busy={vm.isSaving} onClose={() => setSection(null)} onSubmit={vm.save} /> : null}
      {vm.data && importOpen ? (
        <ProfileImportModal
          open
          busy={vm.isSaving}
          onClose={() => setImportOpen(false)}
          onApply={vm.applyImport}
        />
      ) : null}
    </div>
  );
}

function ExperiencesList({ profile, onEdit }: { profile: Profile; onEdit: () => void }) {
  if (profile.experiences.length === 0) return <Vide icon="work_history" title="Votre parcours commence ici" description="Ajoutez une première expérience pour contextualiser vos compétences." action="Ajouter une expérience" onEdit={onEdit} />;
  return (
    <ol>
      {profile.experiences.map((item, index) => (
        <li key={`${item.title}-${index}`} className="border-b border-line px-[18px] py-3.5 last:border-b-0">
          <div className="mb-[3px] flex items-baseline justify-between gap-3">
            <p className="min-w-0 text-item font-semibold text-ink">{item.title}</p>
            <p className="flex-none text-label text-ink-faint">
              {item.start_date} — {item.current ? "Aujourd’hui" : item.end_date ?? "?"}
            </p>
          </div>
          <p className="mb-[7px] text-note font-medium text-accent">
            {item.company}
            {item.location ? ` · ${item.location}` : ""}
          </p>
          {item.description ? (
            <p className="text-label leading-[1.6] text-ink-muted">{item.description}</p>
          ) : null}
        </li>
      ))}
    </ol>
  );
}

function SkillsList({ profile, onEdit }: { profile: Profile; onEdit: () => void }) {
  if (profile.skills.length === 0) return <Vide icon="psychology" title="Aucune compétence ajoutée" description="Commencez par les savoir-faire les plus importants pour le poste visé." action="Ajouter des compétences" onEdit={onEdit} />;
  return (
    <ul className="flex flex-wrap gap-[7px] px-[18px] py-[15px]">
      {profile.skills.map((item, index) => (
        <li
          key={`${item.name}-${index}`}
          className="rounded-pill bg-neutral-tint px-2.5 py-[5px] text-label font-medium text-ink-muted"
        >
          {item.name}
        </li>
      ))}
    </ul>
  );
}

function EducationList({ profile, onEdit }: { profile: Profile; onEdit: () => void }) {
  return <SimpleList items={profile.education.map((item) => ({ title: item.degree, meta: item.school, body: [item.start_date, item.end_date].filter(Boolean).join(" — ") || item.description }))} empty="Aucune formation ajoutée" action="Ajouter une formation" onEdit={onEdit} />;
}

function SimpleList({ items, empty, action, onEdit }: { items: { title: string; meta: string | null; body: string | null }[]; empty: string; action: string; onEdit: () => void }) {
  if (items.length === 0) return <Vide icon="add_notes" title={empty} description="Cette section est facultative, mais peut renforcer votre profil." action={action} onEdit={onEdit} />;
  return <ul className="divide-y divide-line">{items.map((item, index) => <li key={`${item.title}-${index}`} className="px-4 py-3"><div className="flex items-baseline justify-between gap-3"><p className="font-medium text-ink">{item.title}</p>{item.meta ? <span className="text-meta text-ink-faint">{item.meta}</span> : null}</div>{item.body ? <p className="mt-1 text-body text-ink-muted">{item.body}</p> : null}</li>)}</ul>;
}

function Vide({ icon, title, description, action, onEdit }: { icon: IconName; title: string; description: string; action: string; onEdit: () => void }) {
  return <EmptyState icon={icon} title={title} description={description} action={<Button variant="secondary" icon="add" onClick={onEdit}>{action}</Button>} />;
}

function Row({ label, value }: { label: string; value: string | null }) {
  return (
    <div className="flex items-center justify-between gap-3.5 border-b border-line py-[9px] last:border-b-0">
      <span className="flex-none text-note text-ink-faint">{label}</span>
      <span className="min-w-0 flex-1 truncate text-right text-body font-medium text-ink">
        {value || <span className="font-normal text-ink-faint">Non renseigné</span>}
      </span>
    </div>
  );
}
