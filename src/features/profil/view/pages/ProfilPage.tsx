import { useState } from "react";
import { useNavigate } from "react-router-dom";
import { AppError } from "@/shared/types/app-error";
import type { Profil } from "@/shared/types/generated/profil";
import { Button, EmptyState, ErrorBanner, Icon, PageHeader } from "@/shared/ui";
import { useProfilViewModel } from "../../viewmodel/useProfilViewModel";
import { ProfilSectionModal, type ProfilSection } from "../components/ProfilSectionModal";
import { ProfilImportModal } from "../components/ProfilImportModal";
import {
  CompletionRing,
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
      <PageHeader
        icon="account_circle"
        title="Profil et objectif"
        subtitle="Votre parcours, prêt à alimenter vos candidatures"
        secondary={<div className="flex gap-2"><Button variant="secondary" icon="upload_file" onClick={() => setImportOpen(true)}>Importer un CV</Button><Button variant="secondary" icon="description" onClick={() => void navigate("/documents/cv")}>Mes CV</Button></div>}
        primary={<Button variant="primary" icon="edit" disabled={!vm.data} onClick={() => setSection("identite")}>Modifier le profil</Button>}
      />

      <div className="min-h-0 flex-1 overflow-y-auto">
        {vm.isLoading ? (
          <ProfilSkeleton />
        ) : vm.error || !vm.data ? (
          <div className="p-6"><ErrorBanner message={vm.error instanceof AppError ? vm.error.message : "Le profil n’a pas pu être chargé."} onRetry={vm.recharger} /></div>
        ) : (
          <div className="space-y-4 p-5 min-[1200px]:p-6">
            <section className="overflow-hidden rounded-card border border-line bg-surface shadow-e1">
              <div className="grid items-center gap-6 p-5 sm:grid-cols-[minmax(0,1fr)_auto] sm:p-6">
                <ProfileIdentity identite={vm.data.profil.identite} />
                <div className="flex items-center gap-4 border-t border-line pt-5 sm:border-l sm:border-t-0 sm:pl-6 sm:pt-0">
                  <CompletionRing value={vm.data.completion} />
                  <div className="max-w-56">
                    <p className="text-label font-semibold text-ink">Progression du profil</p>
                    <p className="mt-1 text-meta leading-relaxed text-ink-muted">
                      {vm.data.sectionsIncompletes.length === 0
                        ? "Votre profil contient toutes les sections essentielles."
                        : `Prochaine étape : ${vm.data.sectionsIncompletes.slice(0, 2).join(" et ").toLowerCase()}.`}
                    </p>
                  </div>
                </div>
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
            </section>

            <div className="grid items-start gap-4 xl:grid-cols-[minmax(0,1fr)_340px]">
              <div>
                <ProfilPanel tab="experiences" active={tab === "experiences"}>
                  <SectionCard icon="work_history" title="Expériences professionnelles" meta={`${vm.data.profil.experiences.length} entrée${vm.data.profil.experiences.length > 1 ? "s" : ""}`} onEdit={() => setSection("experiences")}>
                    <ExperiencesList profil={vm.data.profil} onEdit={() => setSection("experiences")} />
                  </SectionCard>
                </ProfilPanel>
                <ProfilPanel tab="competences" active={tab === "competences"}>
                  <SectionCard icon="psychology" title="Compétences" meta={`${vm.data.profil.competences.length} ajoutée${vm.data.profil.competences.length > 1 ? "s" : ""}`} onEdit={() => setSection("competences")}>
                    <CompetencesList profil={vm.data.profil} onEdit={() => setSection("competences")} />
                  </SectionCard>
                </ProfilPanel>
                <ProfilPanel tab="formations" active={tab === "formations"}>
                  <div className="space-y-4">
                    <SectionCard icon="school" title="Formations" meta={`${vm.data.profil.formations.length}`} onEdit={() => setSection("formations")}>
                      <FormationsList profil={vm.data.profil} onEdit={() => setSection("formations")} />
                    </SectionCard>
                    <SectionCard icon="rocket_launch" title="Projets" meta={`${vm.data.profil.projets.length}`} onEdit={() => setSection("projets")}>
                      <SimpleList items={vm.data.profil.projets.map((item) => ({ title: item.nom, meta: item.technologies, body: item.description }))} empty="Aucun projet ajouté" action="Ajouter un projet" onEdit={() => setSection("projets")} />
                    </SectionCard>
                    <SectionCard icon="workspace_premium" title="Certifications" meta={`${vm.data.profil.certifications.length}`} onEdit={() => setSection("certifications")}>
                      <SimpleList items={vm.data.profil.certifications.map((item) => ({ title: item.nom, meta: item.organisme, body: item.date }))} empty="Aucune certification ajoutée" action="Ajouter une certification" onEdit={() => setSection("certifications")} />
                    </SectionCard>
                  </div>
                </ProfilPanel>
                <ProfilPanel tab="langues" active={tab === "langues"}>
                  <SectionCard icon="translate" title="Langues" meta={`${vm.data.profil.langues.length}`} onEdit={() => setSection("langues")}>
                    <SimpleList items={vm.data.profil.langues.map((item) => ({ title: item.nom, meta: item.niveau, body: null }))} empty="Aucune langue ajoutée" action="Ajouter une langue" onEdit={() => setSection("langues")} />
                  </SectionCard>
                </ProfilPanel>
              </div>

              <aside className="space-y-4 xl:sticky xl:top-5">
                <section className="rounded-card border border-line bg-surface p-4 shadow-e1">
                  <div className="mb-3 flex items-center gap-2"><Icon name="flag" size={17} className="text-accent" /><h2 className="text-section text-ink">Objectif professionnel</h2></div>
                  {vm.data.profil.identite.titre || vm.data.profil.identite.resume ? (
                    <div className="space-y-2"><p className="font-medium text-ink">{vm.data.profil.identite.titre ?? "Objectif à préciser"}</p>{vm.data.profil.identite.resume ? <p className="text-body leading-relaxed text-ink-muted">{vm.data.profil.identite.resume}</p> : null}</div>
                  ) : (
                    <p className="text-body leading-relaxed text-ink-muted">Ajoutez un poste visé et quelques lignes pour donner une direction claire à votre CV.</p>
                  )}
                  <Button variant="ghost" icon="edit" className="mt-3" onClick={() => setSection("identite")}>Préciser mon objectif</Button>
                </section>

                <section className="rounded-card border border-line bg-surface p-4 shadow-e1">
                  <div className="mb-3 flex items-center gap-2"><Icon name="contact_page" size={17} className="text-accent" /><h2 className="text-section text-ink">Coordonnées</h2></div>
                  <dl className="space-y-2 text-body">
                    <Info icon="mail" label={vm.data.profil.identite.email || "E-mail non renseigné"} />
                    <Info icon="call" label={vm.data.profil.identite.telephone ?? "Téléphone non renseigné"} />
                    <Info icon="location_on" label={vm.data.profil.identite.ville ?? "Ville non renseignée"} />
                  </dl>
                  <p className="mt-4 border-t border-line pt-3 text-meta text-ink-faint">{vm.data.updatedAt ? `Mis à jour le ${formatDate(vm.data.updatedAt)}` : "Profil jamais enregistré"}</p>
                </section>
              </aside>
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
  return <ol className="divide-y divide-line">{profil.experiences.map((item, index) => <li key={`${item.intitule}-${index}`} className="grid gap-2 px-4 py-4 sm:grid-cols-[18px_minmax(0,1fr)_auto]"><span className="mt-1.5 size-2 rounded-full bg-accent ring-4 ring-accent-tint" /><div><p className="font-medium text-ink">{item.intitule}</p><p className="text-body text-ink-muted">{item.entreprise}{item.lieu ? ` · ${item.lieu}` : ""}</p>{item.description ? <p className="mt-2 text-body leading-relaxed text-ink-muted">{item.description}</p> : null}</div><p className="tabular text-meta text-ink-faint">{item.dateDebut} — {item.posteActuel ? "Aujourd’hui" : item.dateFin ?? "?"}</p></li>)}</ol>;
}

function CompetencesList({ profil, onEdit }: { profil: Profil; onEdit: () => void }) {
  if (profil.competences.length === 0) return <Vide icon="psychology" title="Aucune compétence ajoutée" description="Commencez par les savoir-faire les plus importants pour le poste visé." action="Ajouter des compétences" onEdit={onEdit} />;
  return <ul className="flex flex-wrap gap-2 p-4">{profil.competences.map((item, index) => <li key={`${item.nom}-${index}`} className="rounded-full border border-accent-border bg-accent-tint px-3 py-1.5 text-body font-medium text-accent">{item.nom}</li>)}</ul>;
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

function Info({ icon, label }: { icon: string; label: string }) {
  return <div className="flex items-center gap-2"><dt className="sr-only">{icon}</dt><Icon name={icon} size={15} className="text-ink-faint" /><dd className="min-w-0 truncate text-ink-muted">{label}</dd></div>;
}

function formatDate(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? value : new Intl.DateTimeFormat("fr-FR", { day: "2-digit", month: "short", year: "numeric" }).format(date);
}
