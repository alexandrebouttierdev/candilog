import type { ResumeDocument, ResumeWorkspace } from "@/shared/types/generated/documents";
import { Button, ConfirmDialog, ErrorBanner } from "@/shared/ui";
import { useProfilePhoto } from "@/features/profile";
import { updateResumeField } from "../../model/resumeWorkspace";
import { useBaseResumeViewModel } from "../../viewmodel/useBaseResumeViewModel";
import { EmptySheet } from "../components/EmptySheet";
import { GeneratorFrame } from "../components/GeneratorFrame";
import { ProfileGapBanner } from "../components/ProfileGapBanner";
import { ResumePaper } from "../components/ResumePaper";
import { SectionToggles } from "../components/SectionToggles";

/**
 * Enveloppe neutre autour du document composé : `ResumePaper` et `updateResumeField`
 * attendent un `ResumeWorkspace` complet mais n'en lisent jamais que `document` — un CV de
 * base n'a ni offre, ni score, ni proposition. Ces champs ne servent qu'à satisfaire leur
 * type ; ils ne sont jamais affichés ni lus.
 */
function toEditableWorkspace(document: ResumeDocument): ResumeWorkspace {
  return {
    schema_version: 1,
    document,
    job_offer: { title: "", skills: [], soft_skills: [], experience: null, keywords: [], requirements: [], location: null },
    analysis: { recap: "", recommendations: [], content_recommendations: [] },
    score: {
      total: 0,
      skills: null,
      experience: null,
      ats: null,
      present: [],
      missing: [],
      breakdown: [],
      evaluations: [],
      critical_requirements_penalty: 0,
    },
    initial_score: 0,
    proposals: [],
    profile_library: [],
    decisions: { explicitly_added: [], explicitly_removed: [], ignored: [] },
    layout: { status: "available", used_per_mille: 0, remaining_points: 0, page_count: 1, overflow: false },
    content_recommendations: [],
    recommendation_error: null,
  };
}

/** La feuille A4 retouchable d'un CV de base : rendue seulement une fois le document connu. */
function BaseResumeSheet({
  document,
  photo,
  onChange,
}: {
  document: ResumeDocument;
  photo: string | null;
  onChange: (next: ResumeDocument) => void;
}) {
  const workspace = toEditableWorkspace(document);
  return (
    <div className="flex min-h-0 flex-1 flex-col items-center gap-3 overflow-auto p-[26px]">
      <ResumePaper
        workspace={workspace}
        editable
        onChange={(field, value) => onChange(updateResumeField(workspace, field, value).document)}
        photo={photo}
      />
    </div>
  );
}

/**
 * Écran du CV de base (`docs/DESIGN.md` § Générateurs) : composé depuis le seul profil dès
 * l'ouverture, sans offre ni IA. Pas de colonne droite — rien à décider après coup — et pas
 * de `BetaBadge`, qui signale les fonctions passant par l'IA, ce qui n'est pas le cas ici.
 */
export function BaseResumePage() {
  const vm = useBaseResumeViewModel();
  const photo = useProfilePhoto().data ?? null;
  const canSave = Boolean(vm.name.trim()) && vm.document !== null && !vm.isSaving && !vm.isComposing;

  return (
    <GeneratorFrame
      title="CV de base"
      actions={
        <Button variant="primary" size="bar" shortcut="mod+s" disabled={!canSave} onClick={() => void vm.save()}>
          {vm.isSaving ? "Enregistrement…" : "Enregistrer"}
        </Button>
      }
      left={
        <>
          <ProfileGapBanner gaps={vm.profileGaps} what="CV" />
          <SectionToggles
            title="Ce qui figure sur le CV"
            options={vm.options}
            excluded={vm.excluded}
            disabled={vm.isComposing}
            onToggle={vm.toggle}
          />
          {vm.error ? <ErrorBanner title="Composition impossible" message={vm.error} /> : null}
        </>
      }
      status="composé depuis votre profil · aucune offre"
      keys={[{ label: "Enregistrer", shortcut: "mod+s" }]}
      onSave={() => void vm.save()}
    >
      {vm.document ? (
        <BaseResumeSheet document={vm.document} photo={photo} onChange={vm.setDocument} />
      ) : (
        <EmptySheet busy={vm.isComposing} />
      )}
      <ConfirmDialog
        open={vm.pendingToggle !== null}
        register="confirmation"
        title="Recomposer la feuille ?"
        description="Vos retouches sur cette feuille seront perdues."
        confirmLabel="Recomposer"
        cancelLabel="Garder mes retouches"
        onCancel={vm.cancelToggle}
        onConfirm={vm.confirmToggle}
      />
    </GeneratorFrame>
  );
}
