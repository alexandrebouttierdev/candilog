import type {
  ResumeContentRecommendation,
  ResumeLayoutStatus,
  ResumeProfileItem,
  ResumeProposal,
  ResumeWorkspace,
} from "@/shared/types/generated/documents";
import { availableProfileItems, missingProfileSkills } from "../../model/resumeWorkspace";
import { Button, Icon, StatusPill } from "@/shared/ui";

const LAYOUT_LABELS: Record<ResumeLayoutStatus, { label: string; tone: "success" | "neutral" | "warning" | "danger" }> = {
  spacious: { label: "Bonne marge", tone: "success" },
  available: { label: "Espace disponible", tone: "neutral" },
  almost_full: { label: "Peu d’espace restant", tone: "warning" },
  full: { label: "CV presque plein", tone: "warning" },
  overflow: { label: "Dépassement", tone: "danger" },
};

const ITEM_LABELS: Record<ResumeProfileItem["content"]["type"], string> = {
  skill: "Compétences",
  project: "Projets",
  certification: "Certifications",
  language: "Langues",
};

/** Assistant éditorial : bibliothèque réelle du profil et sélection IA restent séparées. */
export function ResumeAtsPanel({
  workspace,
  onAddProfileItem,
  onApplyRecommendation,
  onIgnoreRecommendation,
  onAccept,
  onReject,
  onUndo,
  busy = false,
}: {
  workspace: ResumeWorkspace;
  onAddProfileItem: (itemId: string) => void;
  onApplyRecommendation: (recommendationId: string) => void;
  onIgnoreRecommendation: (recommendationId: string) => void;
  onAccept: (proposalId: string) => void;
  onReject: (proposalId: string) => void;
  onUndo: (proposalId: string) => void;
  busy?: boolean;
}) {
  const layout = LAYOUT_LABELS[workspace.layout.status];
  const available = availableProfileItems(workspace);
  const missing = missingProfileSkills(workspace);
  const hasOffer = Boolean(workspace.job_offer.title.trim()
    || workspace.job_offer.skills.length
    || workspace.job_offer.keywords.length);

  return (
    <div className="space-y-6 p-4">
      <section className="space-y-2" aria-label="Espace disponible dans le CV">
        <div className="flex items-center justify-between gap-3">
          <p className="text-label font-semibold text-ink">Mise en page</p>
          <StatusPill tone={layout.tone} icon={workspace.layout.overflow ? "warning" : "article"}>
            {layout.label}
          </StatusPill>
        </div>
        <div className="h-1.5 overflow-hidden rounded-pill bg-fill" aria-hidden="true">
          <div
            className={`h-full rounded-pill ${workspace.layout.overflow ? "bg-danger" : workspace.layout.status === "almost_full" || workspace.layout.status === "full" ? "bg-warning" : "bg-accent"}`}
            style={{ width: `${Math.min(100, Math.max(4, workspace.layout.used_per_mille / 10))}%` }}
          />
        </div>
        {workspace.layout.overflow ? (
          <p className="text-meta text-danger-text">
            Le CV dépasse la page recommandée. Vos choix sont conservés, mais aucun ajout ne sera recommandé.
          </p>
        ) : null}
      </section>

      <section className="space-y-3" aria-labelledby="recommendations-title">
        <SectionHeading id="recommendations-title" icon="auto_awesome" title="Recommandé pour cette offre" />
        {!hasOffer ? (
          <CompactState icon="target" title="Offre absente" description="Ajoutez une offre pour obtenir une sélection priorisée." />
        ) : workspace.recommendation_error ? (
          <CompactState icon="warning" title="Recommandations IA indisponibles" description="Les contenus du profil restent disponibles et peuvent être ajoutés manuellement." />
        ) : busy ? (
          <CompactState icon="progress_activity" title="Mise à jour en cours" description="La pertinence et la place disponible sont recalculées." />
        ) : workspace.content_recommendations.length === 0 ? (
          <CompactState
            icon={workspace.layout.overflow ? "warning" : "task_alt"}
            title={workspace.layout.overflow || workspace.layout.status === "full" ? "Aucun ajout conseillé" : "Aucune recommandation prioritaire"}
            description={workspace.layout.overflow || workspace.layout.status === "full"
              ? "Le document n’a plus assez de place pour un ajout propre."
              : "Les contenus utiles restent disponibles ci-dessous pour un ajout manuel."}
          />
        ) : (
          <ol className="divide-y divide-line border-y border-line">
            {workspace.content_recommendations.map((recommendation) => (
              <ContentRecommendationRow
                key={recommendation.id}
                recommendation={recommendation}
                workspace={workspace}
                disabled={busy}
                onApply={onApplyRecommendation}
                onIgnore={onIgnoreRecommendation}
              />
            ))}
          </ol>
        )}
      </section>

      <section className="space-y-3" aria-labelledby="available-title">
        <SectionHeading id="available-title" icon="inventory_2" title="Disponible dans votre profil" />
        {available.length === 0 ? (
          <CompactState icon="task_alt" title="Tout est à jour" description="Aucun autre élément du profil n’est disponible." />
        ) : (
          <ProfileLibrary items={available} disabled={busy} onAdd={onAddProfileItem} />
        )}
      </section>

      {missing.length > 0 ? (
        <section className="space-y-3" aria-labelledby="gaps-title">
          <SectionHeading id="gaps-title" icon="warning" title="Compétences manquantes à vérifier" />
          <p className="text-meta leading-relaxed text-ink-faint">
            Demandées par l’offre, mais absentes de votre profil. Ajoutez-les d’abord au profil uniquement si vous les maîtrisez.
          </p>
          <ul className="flex flex-wrap gap-1.5">
            {missing.map((skill) => <li key={skill} className="rounded-tag bg-warning-tint px-2 py-1 text-meta text-warning">{skill}</li>)}
          </ul>
        </section>
      ) : null}

      {workspace.proposals.some((proposal) => proposal.kind === "text_replacement") ? (
        <section className="space-y-3" aria-labelledby="writing-title">
          <SectionHeading id="writing-title" icon="edit_note" title="Optimisations de rédaction" />
          <ul className="divide-y divide-line border-y border-line">
            {workspace.proposals.filter((proposal) => proposal.kind === "text_replacement").map((proposal) => (
              <ResumeProposalRow key={proposal.id} proposal={proposal} busy={busy} onAccept={onAccept} onReject={onReject} onUndo={onUndo} />
            ))}
          </ul>
        </section>
      ) : null}
    </div>
  );
}

type HeadingIcon = "auto_awesome" | "inventory_2" | "warning" | "edit_note";
function SectionHeading({ id, icon, title }: { id: string; icon: HeadingIcon; title: string }) {
  return <div className="flex items-center gap-2"><Icon name={icon} size={16} className="text-accent" /><h3 id={id} className="text-label font-semibold text-ink">{title}</h3></div>;
}

type StateIcon = "target" | "progress_activity" | "warning" | "task_alt";
function CompactState({ icon, title, description }: { icon: StateIcon; title: string; description: string }) {
  return (
    <div className="flex gap-2.5 rounded-card bg-fill p-3">
      <Icon name={icon} size={17} className="mt-0.5 flex-none text-ink-faint" />
      <div><p className="text-label font-medium text-ink">{title}</p><p className="mt-0.5 text-meta leading-relaxed text-ink-faint">{description}</p></div>
    </div>
  );
}

function ContentRecommendationRow({ recommendation, workspace, disabled, onApply, onIgnore }: {
  recommendation: ResumeContentRecommendation;
  workspace: ResumeWorkspace;
  disabled: boolean;
  onApply: (id: string) => void;
  onIgnore: (id: string) => void;
}) {
  const relevance = recommendation.relevance === "very_relevant" ? "Très pertinent" : recommendation.relevance === "relevant" ? "Pertinent" : "Secondaire";
  const replacementAction = recommendation.action.type === "replace" ? recommendation.action : null;
  const replacement = replacementAction !== null;
  const removed = replacementAction ? workspace.profile_library.find((item) => item.id === replacementAction.remove_item_id)?.label : null;
  return (
    <li className="space-y-2.5 py-3 first:pt-2">
      <div className="flex items-start justify-between gap-2"><p className="min-w-0 text-label font-semibold text-ink">{recommendation.label}</p><span className="flex-none text-meta font-medium text-accent">{relevance}</span></div>
      {replacement && removed ? <p className="flex items-center gap-1.5 text-meta text-ink-muted"><span className="line-through">{removed}</span><Icon name="swap_horiz" size={14} /><span>{recommendation.label}</span></p> : null}
      <p className="text-meta leading-relaxed text-ink-muted">{recommendation.reason}</p>
      <div className="flex gap-1.5"><Button size="dialog" variant="primary" disabled={disabled} onClick={() => onApply(recommendation.id)}>{replacement ? "Appliquer" : "Ajouter"}</Button><Button size="dialog" variant="ghost" disabled={disabled} onClick={() => onIgnore(recommendation.id)}>Ignorer</Button></div>
    </li>
  );
}

function ProfileLibrary({ items, disabled, onAdd }: { items: ResumeProfileItem[]; disabled: boolean; onAdd: (id: string) => void }) {
  return (
    <div className="space-y-4">
      {(["skill", "project", "certification", "language"] as const).map((kind) => {
        const group = items.filter((item) => item.content.type === kind);
        if (group.length === 0) return null;
        return <div key={kind}><p className="mb-1.5 text-meta font-semibold uppercase tracking-wide text-ink-faint">{ITEM_LABELS[kind]}</p><ul className="divide-y divide-line border-y border-line">{group.map((item) => <li key={item.id} className="flex items-center gap-2 py-2"><div className="min-w-0 flex-1"><p className="truncate text-label text-ink">{item.label}</p>{item.detail ? <p className="truncate text-meta text-ink-faint">{item.detail}</p> : null}</div><Button size="dialog" variant="ghost" icon="add" disabled={disabled} aria-label={`Ajouter ${item.label}`} onClick={() => onAdd(item.id)}>Ajouter</Button></li>)}</ul></div>;
      })}
    </div>
  );
}

function ResumeProposalRow({ proposal, busy, onAccept, onReject, onUndo }: { proposal: ResumeProposal; busy: boolean; onAccept: (id: string) => void; onReject: (id: string) => void; onUndo: (id: string) => void }) {
  return (
    <li className="space-y-2.5 py-3">
      <div className="flex items-start justify-between gap-2"><p className="text-label font-semibold text-ink">{proposal.label}</p>{!proposal.applicable ? <StatusPill tone="neutral" icon="block">Non applicable</StatusPill> : proposal.status === "accepted" ? <StatusPill tone="success" icon="check">Appliquée</StatusPill> : proposal.status === "rejected" ? <StatusPill tone="neutral" icon="close">Ignorée</StatusPill> : null}</div>
      <p className="text-meta leading-relaxed text-ink-muted">{proposal.proposed_text}</p>
      {proposal.status === "pending" && proposal.applicable ? <div className="flex gap-1.5"><Button size="dialog" variant="primary" disabled={busy} onClick={() => onAccept(proposal.id)}>Accepter</Button><Button size="dialog" variant="ghost" disabled={busy} onClick={() => onReject(proposal.id)}>Ignorer</Button></div> : proposal.status !== "pending" ? <Button size="dialog" variant="ghost" disabled={busy} onClick={() => onUndo(proposal.id)}>Annuler</Button> : null}
    </li>
  );
}
