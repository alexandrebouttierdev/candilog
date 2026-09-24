import { useState } from "react";
import { useLocation } from "react-router-dom";
import { Button, ConfirmDialog, ErrorBanner } from "@/shared/ui";
import { cn } from "@/shared/lib/cn";
import { useLetterWriterViewModel } from "../../viewmodel/useLetterWriterViewModel";
import { useStepLog } from "../../viewmodel/useStepLog";
import { EmptySheet } from "../components/EmptySheet";
import { StopGenerationDialog } from "../components/StopGenerationDialog";
import { GeneratorFrame, PaneSection, RunMeter, StepList } from "../components/GeneratorFrame";
import { OfferSource } from "../components/OfferSource";
import { Segmented } from "../components/Segmented";
import { SectionToggles } from "../components/SectionToggles";
import { LetterEditor } from "../components/LetterEditor";
import type { LetterPaperField } from "../components/LetterPaper";
import { Champ, coverLetterFromNavigation } from "./documentPageSupport";

type Echange = { auteur: "vous" | "candilog"; texte: string };

/** Étapes annoncées par le backend pendant la rédaction d'une lettre. */
const LETTER_STEPS = ["Rédaction", "Relecture du français"] as const;

const TONES = [
  { value: "formal", label: "Formel" },
  { value: "casual", label: "Naturel" },
  { value: "creative", label: "Créatif" },
] as const;

const LENGTHS = [
  { value: "short", label: "Courte" },
  { value: "medium", label: "Moyenne" },
  { value: "long", label: "Longue" },
] as const;

/** Consignes fréquentes, envoyées telles quelles comme correction. */
const QUICK_FIXES = ["Plus court", "Moins formel", "Ajoute un chiffre", "Cite l'entreprise"] as const;

/**
 * Rédacteur de lettre (`screens/16-generator-letter.png`) : surcouche plein écran — offre,
 * destinataire, ton et longueur à gauche, lettre éditable au centre avec la barre de
 * corrections, déroulé et historique à droite. Le brief reste visible : changer le ton ou
 * l'offre puis relancer ne demande aucun aller-retour.
 */
export function LetterWriterPage() {
  const location = useLocation();
  const coverLetterInitiale = coverLetterFromNavigation(location.state);
  const vm = useLetterWriterViewModel(coverLetterInitiale);
  const running = vm.operation !== null;
  const steps = useStepLog(LETTER_STEPS, vm.progress?.step ?? null, running, vm.elapsedMs);
  const hasLetter = vm.output.trim() !== "";
  const corrections = vm.exchanges.filter((exchange: Echange) => exchange.auteur === "vous").length;
  const canSave = hasLetter && !vm.isSaving && !vm.overflow && !running;
  // Avant toute rédaction, une feuille neutre (`screens/16`) ; l'éditeur apparaît avec la
  // lettre, ou tout de suite si l'on préfère l'écrire soi-même.
  const [manual, setManual] = useState(false);
  const [askStop, setAskStop] = useState(false);
  const currentStep = steps.findIndex((step) => step.state === "running") + 1;
  const showEditor = hasLetter || manual;

  const sendInstruction = (instruction: string) => {
    const value = instruction.trim();
    if (value && !running) void vm.run(value);
  };

  return (
    <GeneratorFrame
      title="Générer une lettre"
      actions={
        running ? (
          <Button size="bar" shortcut="mod+." disabled={vm.stopping} onClick={() => setAskStop(true)}>
            {vm.stopping ? "Arrêt…" : "Arrêter"}
          </Button>
        ) : hasLetter ? (
          <>
            <Button variant="ghost" size="bar" onClick={() => vm.setAbandonOpen(true)}>
              Abandonner…
            </Button>
            <Button size="bar" onClick={() => void vm.correct()}>
              Corriger l’orthographe
            </Button>
            <Button size="bar" disabled={vm.overflow} onClick={() => void vm.exportPdf()}>
              Exporter en PDF
            </Button>
            <Button variant="primary" size="bar" shortcut="mod+s" disabled={!canSave} onClick={() => vm.save()}>
              {vm.isSaving ? "Enregistrement…" : "Enregistrer"}
            </Button>
          </>
        ) : (
          <Button variant="primary" size="bar" shortcut="mod+enter" onClick={() => void vm.run(null)}>
            Rédiger la lettre
          </Button>
        )
      }
      left={
        <>
          <PaneSection title="Offre visée">
            <OfferSource
              value={vm.context}
              onChange={vm.setContext}
              readClipboard={vm.readClipboard}
              disabled={running}
              textLabel="Contexte ou offre"
              required={false}
              onPick={(application) => {
                vm.setCompany(application.company_name ?? "");
                vm.setJobTitle(application.job_title);
              }}
            />
          </PaneSection>
          <PaneSection title="Entreprise et poste">
            <div className="flex flex-col gap-2.5">
              <Champ label="Entreprise" value={vm.company} onChange={vm.setCompany} />
              <Champ label="Poste ciblé" value={vm.jobTitle} onChange={vm.setJobTitle} />
            </div>
          </PaneSection>
          <PaneSection title="Destinataire">
            <Champ label="Nom du recruteur — facultatif" value={vm.recipient} onChange={vm.setRecipient} />
            <p className="mt-1.5 text-tiny leading-[1.45] text-tx-5">
              Sans nom, la lettre utilisera « Madame, Monsieur ».
            </p>
          </PaneSection>
          <SectionToggles
            title="Arguments autorisés"
            options={vm.argumentOptions}
            excluded={vm.excludedSections}
            disabled={running}
            onToggle={vm.toggleSection}
          />
          <PaneSection title="Ton">
            <Segmented label="Ton" value={vm.tone} options={TONES} disabled={running} onChange={vm.setTone} />
          </PaneSection>
          <PaneSection title="Longueur">
            <Segmented label="Longueur" value={vm.length} options={LENGTHS} disabled={running} onChange={vm.setLength} />
          </PaneSection>
          {hasLetter && !running ? (
            <Button className="w-full" size="compact" onClick={() => void vm.run(null)}>
              Rédiger une nouvelle lettre
            </Button>
          ) : null}
          {vm.error ? (
            <div className="mt-3">
              <ErrorBanner title="Rédaction impossible" message={vm.error} />
            </div>
          ) : null}
        </>
      }
      right={
        <>
          <PaneSection title={running ? "En cours" : hasLetter ? "Terminé" : "Étapes"}>
            <StepList steps={steps} />
            {running ? <RunMeter steps={steps} elapsedMs={vm.elapsedMs} tokens={vm.progress?.tokens_used ?? null} /> : null}
            {!running && !hasLetter ? (
              <p className="mt-5 text-small leading-[1.55] text-tx-4">
                Candilog rédige à partir de l'offre et de votre profil, puis relit le français. Vous corrigez ensuite
                en quelques mots — rien n'est envoyé à l'entreprise.
              </p>
            ) : null}
          </PaneSection>
          {vm.exchanges.length > 0 ? (
            <PaneSection title="Historique" aside={`${corrections} correction${corrections > 1 ? "s" : ""}`}>
              <ol className="flex flex-col gap-1.5">
                {vm.exchanges.map((exchange: Echange, index: number) => (
                  <li
                    key={`${exchange.auteur}-${index}`}
                    className={cn(
                      "rounded-r7 px-2.5 py-1.5 text-small leading-[1.45]",
                      exchange.auteur === "vous" ? "bg-sel text-tx-2" : "bg-group text-tx-4",
                    )}
                  >
                    {exchange.texte}
                  </li>
                ))}
              </ol>
            </PaneSection>
          ) : null}
        </>
      }
      status={
        running
          ? `rédaction en cours · étape ${Math.max(currentStep, 1)} / ${steps.length}`
          : hasLetter
            ? `lettre prête · ${corrections} correction${corrections > 1 ? "s" : ""}${vm.overflow ? " · trop longue pour une page" : ""}`
            : "renseignez l'offre, puis rédigez"
      }
      keys={
        running
          ? [{ label: "Arrêter", shortcut: "mod+." }]
          : hasLetter
            ? [{ label: "Enregistrer", shortcut: "mod+s" }]
            : [{ label: "Rédiger", shortcut: "mod+enter" }]
      }
      {...(running ? { onStop: () => setAskStop(true) } : {})}
      closeDisabled={running}
      onSave={() => {
        if (canSave) vm.save();
      }}
      {...(!hasLetter && !running ? { onSubmit: () => void vm.run(null) } : {})}
    >
      {showEditor ? null : (
        <div className="flex min-h-0 flex-1 flex-col overflow-auto">
          <EmptySheet busy={running} />
          {running ? null : (
            <p className="-mt-2 pb-5 text-center">
              <Button variant="link" onClick={() => setManual(true)}>
                Écrire la lettre moi-même
              </Button>
            </p>
          )}
        </div>
      )}
      {showEditor ? (
      <div className="min-h-0 flex-1 overflow-auto p-4">
        <LetterEditor
          value={vm.output}
          readOnly={running}
          identity={vm.identity}
          onSaveIdentity={vm.saveIdentity}
          fields={{
            company: vm.company,
            job_title: vm.jobTitle,
            recipient: vm.recipient,
            recipient_address: vm.recipientAddress,
            job_reference: vm.jobReference,
          }}
          onChange={vm.setOutput}
          onFieldsChange={(field: LetterPaperField, value: string) => {
            if (field === "company") vm.setCompany(value);
            else if (field === "job_title") vm.setJobTitle(value);
            else if (field === "recipient") vm.setRecipient(value);
            else if (field === "recipient_address") vm.setRecipientAddress(value);
            else vm.setJobReference(value);
          }}
          onOverflowChange={vm.setOverflow}
        />
      </div>
      ) : null}
      {hasLetter ? (
        <section aria-label="Corrections" className="flex-none border-t border-bd-soft bg-app px-4 pt-2.5 pb-3">
          <div className="mb-1.5 flex items-baseline gap-2">
            <h2 className="caps">Corrections</h2>
            <span className="font-mono text-caps text-tx-6">
              {corrections} échange{corrections > 1 ? "s" : ""}
            </span>
            <span className="ml-auto text-tiny text-tx-5">Dites ce que vous voulez changer, en français courant.</span>
          </div>
          <form
            className="flex gap-2"
            onSubmit={(event) => {
              event.preventDefault();
              sendInstruction(vm.instruction);
            }}
          >
            <input
              aria-label="Que faut-il changer ?"
              value={vm.instruction}
              disabled={running}
              placeholder="Raccourcis le deuxième paragraphe…"
              onChange={(event) => vm.setInstruction(event.target.value)}
              className="h-8 min-w-0 flex-1 rounded-r7 border border-bd-menu bg-panel px-2.5 text-ui text-tx outline-none placeholder:text-tx-6 focus:border-ac"
            />
            <Button type="submit" size="compact" disabled={running || vm.instruction.trim() === ""}>
              Envoyer
            </Button>
          </form>
          <div className="mt-2 flex flex-wrap gap-1.5">
            {QUICK_FIXES.map((fix) => (
              <button
                key={fix}
                type="button"
                disabled={running}
                onClick={() => sendInstruction(fix)}
                className="h-6 rounded-r6 bg-chip px-2 text-small text-tx-3 hover:text-tx disabled:opacity-50"
              >
                {fix}
              </button>
            ))}
          </div>
        </section>
      ) : null}
      <StopGenerationDialog
        open={askStop && running && !vm.stopping}
        step={vm.progress?.step ?? null}
        elapsedMs={vm.elapsedMs}
        onKeepGoing={() => setAskStop(false)}
        onStop={() => {
          setAskStop(false);
          void vm.stop();
        }}
      />
      <ConfirmDialog
        open={vm.abandonOpen}
        title="Abandonner cette lettre ?"
        description="La lettre affichée et les consignes de l'itération seront perdues."
        note="Le brief, lui, est conservé : vous pourrez relancer une rédaction."
        confirmLabel="Abandonner"
        onCancel={() => vm.setAbandonOpen(false)}
        onConfirm={() => {
          setManual(false);
          vm.abandon();
        }}
      />
    </GeneratorFrame>
  );
}
