import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { AppError } from "@/shared/types/app-error";
import type { Profil } from "@/shared/types/generated/profil";
import { ContextBarAccessory, ContextNote } from "@/app/layout/ContextBar";
import { Button, Card, CardHeader, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { useProfilViewModel } from "../../viewmodel/useProfilViewModel";
import { ProfilSectionModal, type ProfilSection } from "../components/ProfilSectionModal";
import { ProfilImportModal } from "../components/ProfilImportModal";
import {
  CompletionBar,
  ProfileIdentity,
  ProfilPanel,
  ProfilSkeleton,
  ProfilTabs,
  SectionCard,
  type ProfilTab,
} from "../components/ProfilUi";

/** Profil professionnel, objectif et parcours exploités par le générateur de CV. */
export function ProfilPage() {
  const vm = useProfilViewModel();
  const navigate = useNavigate();
  const [tab, setTab] = useState<ProfilTab>("experiences");
  const [section, setSection] = useState<ProfilSection | null>(null);
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
          <Button variant="primary" icon="edit" disabled={!vm.data} onClick={() => setSection("identite")}>
            Modifier le profil
          </Button>
        }
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <ProfilSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="p-6"><ErrorBanner message={vm.error instanceof AppError ? vm.error.message : "Le profil n’a pas pu être chargé."} onRetry={vm.recharger} /></div>
        ) : (
          <div>
            <div className="border-b border-line bg-surface px-7 pt-[22px]">
              <div className="flex flex-wrap items-start gap-4">
                <ProfileIdentity identite={vm.data.profil.identite} />
                <CompletionBar
                  value={vm.data.completion}
                  hint={
                    vm.data.sectionsIncompletes.length === 0
                      ? "Votre profil contient toutes les sections essentielles."
                      : `Ajoutez ${vm.data.sectionsIncompletes.slice(0, 2).join(" et ").toLowerCase()} pour atteindre 100 %.`
                  }
                />
              </div>
              <ProfilTabs
                active={tab}
                onChange={setTab}
                counts={{
                  experiences: vm.data.profil.experiences.length,
                  competences: vm.data.profil.competences.length,
                  formations: vm.data.profil.formations.length,
                  langues: vm.data.profil.langues.length,
                }}
              />
            </div>

            <div className="flex flex-wrap items-start gap-4 px-7 pt-5 pb-8">
              <div className="flex min-w-0 flex-[1_1_460px] flex-col gap-4">
                <ProfilPanel tab="experiences" active={tab === "experiences"}>
                  <SectionCard icon="work_history" title="Expériences" onEdit={() => setSection("experiences")}>
                    <ExperiencesList profil={vm.data.profil} onEdit={() => setSection("experiences")} />
                  </SectionCard>
                </ProfilPanel>
                <ProfilPanel tab="competences" active={tab === "competences"}>
                  <SectionCard icon="psychology" title="Compétences" actionLabel="Gérer" onEdit={() => setSection("competences")}>
                    <CompetencesList profil={vm.data.profil} onEdit={() => setSection("competences")} />
                  </SectionCard>
                </ProfilPanel>
                <ProfilPanel tab="formations" active={tab === "formations"}>
                  <div className="flex flex-col gap-4">
                    <SectionCard icon="school" title="Formations" onEdit={() => setSection("formations")}>
                      <FormationsList profil={vm.data.profil} onEdit={() => setSection("formations")} />
                    </SectionCard>
                    <SectionCard icon="rocket_launch" title="Projets" onEdit={() => setSection("projets")}>
                      <SimpleList items={vm.data.profil.projets.map((item) => ({ title: item.nom, meta: item.technologies, body: item.description }))} empty="Aucun projet ajouté" action="Ajouter un projet" onEdit={() => setSection("projets")} />
                    </SectionCard>
                    <SectionCard icon="workspace_premium" title="Certifications" onEdit={() => setSection("certifications")}>
                      <SimpleList items={vm.data.profil.certifications.map((item) => ({ title: item.nom, meta: item.organisme, body: item.date }))} empty="Aucune certification ajoutée" action="Ajouter une certification" onEdit={() => setSection("certifications")} />
                    </SectionCard>
                  </div>
                </ProfilPanel>
                <ProfilPanel tab="langues" active={tab === "langues"}>
                  <SectionCard icon="translate" title="Langues" onEdit={() => setSection("langues")}>
                    <SimpleList items={vm.data.profil.langues.map((item) => ({ title: item.nom, meta: item.niveau, body: null }))} empty="Aucune langue ajoutée" action="Ajouter une langue" onEdit={() => setSection("langues")} />
                  </SectionCard>
                </ProfilPanel>
              </div>

              <div className="flex max-w-[380px] min-w-0 flex-[1_1_300px] flex-col gap-4">
                <Card clipped>
                  <CardHeader
                    compact
                    icon="badge"
                    meta={
                      <button
                        type="button"
                        onClick={() => setSection("identite")}
                        className="inline-flex items-center gap-[5px] text-label font-medium text-accent hover:opacity-80"
                      >
                        Modifier
                      </button>
                    }
                  >
                    Identité
                  </CardHeader>
                  <div className="px-[18px] pt-1 pb-3">
                    <Ligne label="E-mail" valeur={vm.data.profil.identite.email} />
                    <Ligne label="Téléphone" valeur={vm.data.profil.identite.telephone} />
                    <Ligne label="Ville" valeur={vm.data.profil.identite.ville} />
                    <Ligne label="LinkedIn" valeur={vm.data.profil.identite.linkedin} />
                    <Ligne label="GitHub" valeur={vm.data.profil.identite.github} />
                    <Ligne label="Site" valeur={vm.data.profil.identite.siteWeb} />
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

      {vm.data && section ? <ProfilSectionModal key={section} section={section} profil={vm.data.profil} busy={vm.isSaving} onClose={() => setSection(null)} onSubmit={vm.enregistrer} /> : null}
      {vm.data ? <ProfilImportModal open={importOpen} profil={vm.data.profil} busy={vm.isSaving} onClose={() => setImportOpen(false)} onSubmit={vm.enregistrer} /> : null}
    </div>
  );
}

function ExperiencesList({ profil, onEdit }: { profil: Profil; onEdit: () => void }) {
  if (profil.experiences.length === 0) return <Vide icon="work_history" title="Votre parcours commence ici" description="Ajoutez une première expérience pour contextualiser vos compétences." action="Ajouter une expérience" onEdit={onEdit} />;
  return (
    <ol>
      {profil.experiences.map((item, index) => (
        <li key={`${item.intitule}-${index}`} className="border-b border-line px-[18px] py-3.5 last:border-b-0">
          <div className="mb-[3px] flex items-baseline justify-between gap-3">
            <p className="min-w-0 text-item font-semibold text-ink">{item.intitule}</p>
            <p className="flex-none text-label text-ink-faint">
              {item.dateDebut} — {item.posteActuel ? "Aujourd’hui" : item.dateFin ?? "?"}
            </p>
          </div>
          <p className="mb-[7px] text-note font-medium text-accent">
            {item.entreprise}
            {item.lieu ? ` · ${item.lieu}` : ""}
          </p>
          {item.description ? (
            <p className="text-label leading-[1.6] text-ink-muted">{item.description}</p>
          ) : null}
        </li>
      ))}
    </ol>
  );
}

function CompetencesList({ profil, onEdit }: { profil: Profil; onEdit: () => void }) {
  if (profil.competences.length === 0) return <Vide icon="psychology" title="Aucune compétence ajoutée" description="Commencez par les savoir-faire les plus importants pour le poste visé." action="Ajouter des compétences" onEdit={onEdit} />;
  return (
    <ul className="flex flex-wrap gap-[7px] px-[18px] py-[15px]">
      {profil.competences.map((item, index) => (
        <li
          key={`${item.nom}-${index}`}
          className="rounded-pill bg-neutral-tint px-2.5 py-[5px] text-label font-medium text-ink-muted"
        >
          {item.nom}
        </li>
      ))}
    </ul>
  );
}

function FormationsList({ profil, onEdit }: { profil: Profil; onEdit: () => void }) {
  return <SimpleList items={profil.formations.map((item) => ({ title: item.diplome, meta: item.etablissement, body: [item.dateDebut, item.dateFin].filter(Boolean).join(" — ") || item.description }))} empty="Aucune formation ajoutée" action="Ajouter une formation" onEdit={onEdit} />;
}

function SimpleList({ items, empty, action, onEdit }: { items: { title: string; meta: string | null; body: string | null }[]; empty: string; action: string; onEdit: () => void }) {
  if (items.length === 0) return <Vide icon="add_notes" title={empty} description="Cette section est facultative, mais peut renforcer votre profil." action={action} onEdit={onEdit} />;
  return <ul className="divide-y divide-line">{items.map((item, index) => <li key={`${item.title}-${index}`} className="px-4 py-3"><div className="flex items-baseline justify-between gap-3"><p className="font-medium text-ink">{item.title}</p>{item.meta ? <span className="text-meta text-ink-faint">{item.meta}</span> : null}</div>{item.body ? <p className="mt-1 text-body text-ink-muted">{item.body}</p> : null}</li>)}</ul>;
}

function Vide({ icon, title, description, action, onEdit }: { icon: string; title: string; description: string; action: string; onEdit: () => void }) {
  return <EmptyState icon={icon} title={title} description={description} action={<Button variant="secondary" icon="add" onClick={onEdit}>{action}</Button>} />;
}

function Ligne({ label, valeur }: { label: string; valeur: string | null }) {
  return (
    <div className="flex items-center justify-between gap-3.5 border-b border-line py-[9px] last:border-b-0">
      <span className="flex-none text-note text-ink-faint">{label}</span>
      <span className="min-w-0 flex-1 truncate text-right text-body font-medium text-ink">
        {valeur || <span className="font-normal text-ink-faint">Non renseigné</span>}
      </span>
    </div>
  );
}
