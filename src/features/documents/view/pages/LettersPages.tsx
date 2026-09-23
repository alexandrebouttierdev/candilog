import type { ReactNode } from "react";
import { useLocation } from "react-router-dom";
import { AiStopButton } from "@/features/ai";
import {
  Button,
  ConfirmDialog,
  ErrorBanner,
  FormField,
  Icon,
  PageHeader,
  Select,
  TextArea,
} from "@/shared/ui";
import { useLetterWriterViewModel } from "../../viewmodel/useLetterWriterViewModel";
import { AiProgress, DocumentPanel } from "../components/DocumentUi";
import { LetterEditor } from "../components/LetterEditor";
import type { LetterPaperField } from "../components/LetterPaper";
import {
  Champ,
  ChampOffre,
  Screen,
  coverLetterFromNavigation,
} from "./documentPageSupport";

type Echange = { auteur: "vous" | "candilog"; texte: string };

/**
 * Suite du travail une fois la première lettre écrite.
 *
 * Le brief n'a plus rien à demander à ce stade : ce qui reste à faire, c'est demander des
 * ajustements. Les consignes s'accumulent — « plus court » puis « plus formel » doivent
 * valoir ensemble — et chaque régénération journalise le temps qu'elle a pris, à l'endroit
 * où l'utilisateur regarde.
 */
function IterationPanel({
  echanges,
  consigne,
  busy,
  stopping,
  error,
  progress,
  onConsigneChange,
  onSubmit,
  onCancel,
  onReopenBrief,
}: {
  echanges: Echange[];
  consigne: string;
  busy: boolean;
  stopping: boolean;
  error: string | null;
  progress: ReactNode;
  onConsigneChange: (value: string) => void;
  onSubmit: () => void;
  onCancel: () => void;
  onReopenBrief: () => void;
}) {
  return (
    <DocumentPanel title="Itérations" icon="forum" className="flex min-h-0 flex-col">
      <div className="flex min-h-0 flex-1 flex-col">
        <ol className="min-h-0 flex-1 space-y-2 overflow-y-auto p-4">
          {echanges.map((echange, index) => (
            <li
              key={`${echange.auteur}-${index}`}
              className={echange.auteur === "vous" ? "flex justify-end" : "flex justify-start"}
            >
              <span
                className={
                  echange.auteur === "vous"
                    ? "max-w-[85%] rounded-card bg-accent-tint px-3 py-2 text-body text-ink"
                    : "inline-flex max-w-[85%] items-center gap-1.5 rounded-card bg-fill px-3 py-2 text-meta text-ink-muted"
                }
              >
                {echange.auteur === "candilog" ? <Icon name="schedule" size={14} className="flex-none" /> : null}
                {echange.texte}
              </span>
            </li>
          ))}
        </ol>
        <div className="space-y-3 border-t border-line p-4">
          {error ? <ErrorBanner title="Rédaction impossible" message={error} /> : null}
          {busy ? (
            <>
              {progress}
              <AiStopButton stopping={stopping} onStop={onCancel} />
            </>
          ) : (
            <>
              <FormField label="Que faut-il changer ?">
                {(props) => (
                  <TextArea
                    {...props}
                    rows={3}
                    value={consigne}
                    placeholder="Ex. « Mets en avant ma dernière expérience » ou « Va droit au but »"
                    onChange={(event) => onConsigneChange(event.target.value)}
                  />
                )}
              </FormField>
              <Button
                variant="primary"
                icon="auto_awesome"
                className="w-full"
                disabled={consigne.trim() === ""}
                onClick={onSubmit}
              >
                Régénérer avec cette consigne
              </Button>
              <Button variant="ghost" icon="target" className="w-full" onClick={onReopenBrief}>
                Revenir au brief
              </Button>
            </>
          )}
        </div>
      </div>
    </DocumentPanel>
  );
}

export function LetterWriterPage() {
  const location = useLocation();
  const coverLetterInitiale = coverLetterFromNavigation(location.state);
  const vm = useLetterWriterViewModel(coverLetterInitiale);

  return (
    <Screen
      padded={false}
      header={
        <PageHeader
          icon="edit_note"
          title="Lettre de motivation"
          subtitle="Rédigez, itérez et enregistrez"
          secondary={
            vm.output ? (
              <>
                <Button
                  icon={vm.operation?.kind === "correction" ? "progress_activity" : "edit_note"}
                  disabled={vm.operation !== null}
                  onClick={() => void vm.correct()}
                >
                  {vm.operation?.kind === "correction"
                    ? "Correction en cours…"
                    : "Corriger l’orthographe"}
                </Button>
                <Button
                  icon="download"
                  disabled={vm.overflow || vm.operation !== null}
                  onClick={() => void vm.exportPdf()}
                >
                  Exporter le PDF
                </Button>
                <Button
                  icon="close"
                  disabled={vm.operation !== null}
                  onClick={() => vm.setAbandonOpen(true)}
                >
                  Annuler
                </Button>
              </>
            ) : undefined
          }
          primary={
            vm.output ? (
              <Button
                variant="primary"
                icon={vm.isSaving ? "progress_activity" : "save"}
                disabled={vm.isSaving || vm.overflow || vm.operation !== null}
                onClick={() => vm.save()}
              >
                {vm.isSaving ? "Enregistrement…" : "Enregistrer"}
              </Button>
            ) : undefined
          }
        />
      }
    >
      <div className="grid min-h-0 flex-1 gap-4 overflow-hidden p-5 min-[1200px]:p-6 xl:grid-cols-[350px_minmax(480px,1fr)]">
        {vm.inIteration ? (
          <IterationPanel
            echanges={vm.exchanges}
            consigne={vm.instruction}
            busy={vm.operation !== null}
            stopping={vm.stopping}
            error={vm.error}
            progress={
              vm.operation && !vm.stopping ? (
                <AiProgress progress={vm.progress} elapsedMs={vm.elapsedMs} />
              ) : null
            }
            onConsigneChange={vm.setInstruction}
            onSubmit={() => void vm.run(vm.instruction.trim())}
            onCancel={() => void vm.stop()}
            onReopenBrief={() => vm.setBriefOpen(true)}
          />
        ) : (
          <DocumentPanel title="Brief de rédaction" icon="target" className="flex min-h-0 flex-col">
            <div className="min-h-0 flex-1 space-y-4 overflow-y-auto p-4">
              <Champ label="Entreprise" value={vm.company} onChange={vm.setCompany} />
              <Champ label="Poste ciblé" value={vm.jobTitle} onChange={vm.setJobTitle} />
              <div className="grid grid-cols-2 gap-3">
                <FormField label="Ton">
                  {(props) => (
                    <Select
                      {...props}
                      value={vm.tone}
                      onChange={(e) => vm.setTone(e.target.value)}
                    >
                      <option value="formal">Formel</option>
                      <option value="casual">Naturel</option>
                      <option value="creative">Créatif</option>
                    </Select>
                  )}
                </FormField>
                <FormField label="Longueur">
                  {(props) => (
                    <Select
                      {...props}
                      value={vm.length}
                      onChange={(e) => vm.setLength(e.target.value)}
                    >
                      <option value="short">Courte</option>
                      <option value="medium">Moyenne</option>
                      <option value="long">Longue</option>
                    </Select>
                  )}
                </FormField>
              </div>
              <ChampOffre
                label="Contexte ou offre"
                rows={10}
                value={vm.context}
                onChange={vm.setContext}
                readClipboard={vm.readClipboard}
              />
              {vm.error ? <ErrorBanner title="Rédaction impossible" message={vm.error} /> : null}
              {vm.operation ? (
                <>
                  {vm.stopping ? null : (
                    <AiProgress progress={vm.progress} elapsedMs={vm.elapsedMs} />
                  )}
                  <AiStopButton stopping={vm.stopping} onStop={() => void vm.stop()} />
                </>
              ) : (
                <div className="flex flex-col gap-2">
                  <Button
                    variant="primary"
                    icon="auto_awesome"
                    className="w-full"
                    onClick={() => void vm.run(null)}
                  >
                    {vm.exchanges.length > 0 ? "Rédiger une nouvelle lettre" : "Rédiger la lettre"}
                  </Button>
                  {vm.exchanges.length > 0 ? (
                    <Button
                      variant="ghost"
                      icon="forum"
                      className="w-full"
                      onClick={() => vm.setBriefOpen(false)}
                    >
                      Revenir aux itérations
                    </Button>
                  ) : null}
                </div>
              )}
            </div>
          </DocumentPanel>
        )}
        <LetterEditor
          value={vm.output}
          readOnly={vm.operation !== null}
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
      <ConfirmDialog
        open={vm.abandonOpen}
        title="Abandonner cette lettre ?"
        description="La lettre affichée et les consignes de l'itération seront perdues."
        note="Le brief, lui, est conservé : vous pourrez relancer une rédaction."
        confirmLabel="Abandonner"
        onCancel={() => vm.setAbandonOpen(false)}
        onConfirm={vm.abandon}
      />
    </Screen>
  );
}
