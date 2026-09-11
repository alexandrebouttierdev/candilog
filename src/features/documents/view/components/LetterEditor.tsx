import { useEffect, useRef, useState } from "react";
import { IconButton, Select } from "@/shared/ui";
import { DocumentPanel } from "./DocumentUi";
import { LetterPaper, type LetterPaperField, type LetterPaperFields } from "./LetterPaper";
import type { Identity } from "@/shared/types/generated/profile";
import type { IconName } from "@/shared/ui/icon-names";
import {
  markupFromDom,
  parseLetter,
  toEditableHtml,
  type LetterAlign,
  type LetterSize,
} from "../../model/letterMarkup";

const ALIGNMENTS: { value: LetterAlign; icon: IconName; label: string }[] = [
  { value: "left", icon: "format_align_left", label: "Aligner à gauche" },
  { value: "center", icon: "format_align_center", label: "Centrer" },
  { value: "right", icon: "format_align_right", label: "Aligner à droite" },
];

/**
 * Éditeur de la lettre.
 *
 * La barre d'outils vit dans l'en-tête du panneau, **hors de la feuille** : posée sur le
 * papier elle défilait avec le texte et prenait la place de la lettre. Elle ne propose que
 * ce que l'export PDF sait honorer — gras, souligné, size et alignment — parce qu'un
 * bouton dont l'effet disparaît à l'impression est un piège.
 *
 * Le contenu vit dans le DOM de la zone éditable et n'est relu qu'à la frappe : réinjecter
 * le balisage à chaque rendu replacerait le curseur au début à chaque lettre tapée.
 */
export function LetterEditor({
  value,
  readOnly,
  fields,
  identity = null,
  onSaveIdentity,
  onChange,
  onFieldsChange,
  onOverflowChange,
}: {
  value: string;
  readOnly: boolean;
  fields: LetterPaperFields;
  identity?: Identity | null;
  onSaveIdentity?: (identity: Identity) => void | Promise<unknown>;
  onChange: (markup: string) => void;
  onFieldsChange: (field: LetterPaperField, value: string) => void;
  onOverflowChange: (overflow: boolean) => void;
}) {
  const zone = useRef<HTMLDivElement | null>(null);
  const lastMarkup = useRef<string>("");
  // Dernière sélection connue **dans la lettre** : ouvrir la liste des tailles déplace le
  // curseur hors de la zone, et sans ce repère la commande ne saurait plus sur quoi agir.
  const savedRange = useRef<Range | null>(null);
  const [marks, setMarks] = useState({ bold: false, underline: false });

  useEffect(() => {
    const root = zone.current;
    if (!root || value === lastMarkup.current) return;
    root.innerHTML = toEditableHtml(parseLetter(value));
    lastMarkup.current = value;
  }, [value]);

  useEffect(() => {
    const refreshMarks = () => {
      const root = zone.current;
      const selection = document.getSelection();
      if (!root || !selection?.anchorNode || !root.contains(selection.anchorNode)) return;
      if (selection.rangeCount > 0) savedRange.current = selection.getRangeAt(0).cloneRange();
      setMarks({ bold: queryCommandActive("bold"), underline: queryCommandActive("underline") });
    };
    document.addEventListener("selectionchange", refreshMarks);
    return () => document.removeEventListener("selectionchange", refreshMarks);
  }, []);

  const syncMarkup = () => {
    const root = zone.current;
    if (!root) return;
    const markup = markupFromDom(root);
    lastMarkup.current = markup;
    onChange(markup);
  };

  const applyMark = (command: "bold" | "underline") => {
    restoreSelection();
    runCommand(command);
    setMarks({ bold: queryCommandActive("bold"), underline: queryCommandActive("underline") });
    syncMarkup();
  };

  const forSelectedParagraphs = (action: (element: HTMLElement) => void) => {
    const root = zone.current;
    if (!root) return;
    restoreSelection();
    for (const paragraph of selectedParagraphs(root, savedRange.current)) action(paragraph);
    syncMarkup();
  };

  /** Redonne le curseur à la lettre, là où il était avant le clic sur la barre d'outils. */
  const restoreSelection = () => {
    const root = zone.current;
    if (!root) return;
    root.focus();
    const saved = savedRange.current;
    if (!saved || !root.contains(saved.commonAncestorContainer)) return;
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(saved);
  };

  const barre = (
    <div
      className="flex items-center gap-1"
      onMouseDown={(event) => {
        if (!(event.target instanceof HTMLSelectElement)) event.preventDefault();
      }}
    >
      <IconButton
        icon="format_bold"
        label="Gras"
        aria-pressed={marks.bold}
        disabled={readOnly}
        className={marks.bold ? "bg-accent-tint text-accent" : undefined}
        onClick={() => applyMark("bold")}
      />
      <IconButton
        icon="format_underlined"
        label="Souligné"
        aria-pressed={marks.underline}
        disabled={readOnly}
        className={marks.underline ? "bg-accent-tint text-accent" : undefined}
        onClick={() => applyMark("underline")}
      />
      <span aria-hidden className="mx-1 h-4 w-px bg-line" />
      {ALIGNMENTS.map((alignment) => (
        <IconButton
          key={alignment.value}
          icon={alignment.icon}
          label={alignment.label}
          disabled={readOnly}
          onClick={() =>
            forSelectedParagraphs((element) => {
              element.style.textAlign = alignment.value === "left" ? "" : alignment.value;
            })
          }
        />
      ))}
      <span aria-hidden className="mx-1 h-4 w-px bg-line" />
      <Select
        aria-label="Taille du texte"
        disabled={readOnly}
        className="h-control w-[118px]"
        defaultValue="normal"
        onChange={(event) => {
          const size = event.target.value as LetterSize;
          forSelectedParagraphs((element) => {
            if (size === "normal") delete element.dataset["size"];
            else element.dataset["size"] = size;
          });
        }}
      >
        <option value="small">Petite</option>
        <option value="normal">Normale</option>
        <option value="large">Grande</option>
      </Select>
    </div>
  );

  return (
    <DocumentPanel title="Document" icon="draft" action={barre} className="flex min-h-0 flex-col">
      <div className="flex min-h-0 flex-1 flex-col items-center gap-3 overflow-y-auto bg-page p-[26px]">
        <LetterPaper
          fields={fields}
          identity={identity}
          editable={!readOnly}
          onChange={onFieldsChange}
          onOverflowChange={onOverflowChange}
          {...(onSaveIdentity ? { onSaveIdentity } : {})}
        >
          <div
            ref={zone}
            role="textbox"
            aria-multiline="true"
            aria-label="Contenu de la lettre"
            contentEditable={!readOnly}
            suppressContentEditableWarning
            data-placeholder="La lettre apparaîtra ici après la rédaction. Vous pouvez aussi l'écrire directement."
            onInput={syncMarkup}
            onBlur={syncMarkup}
            onPaste={(event) => {
              // Un collage extérieur arrive avec ses propres styles : seul son texte entre.
              event.preventDefault();
              runCommand("insertText", event.clipboardData.getData("text/plain"));
              syncMarkup();
            }}
            className="letter-body outline-none"
          />
        </LetterPaper>
      </div>
    </DocumentPanel>
  );
}

/** `execCommand` reste le seul moyen d'éditer une sélection sans embarquer un éditeur tiers. */
function runCommand(command: string, value?: string): void {
  try {
    document.execCommand(command, false, value);
  } catch {
    // Moteur sans support : la frappe directe reste possible, la mise en forme non.
  }
}

function queryCommandActive(command: string): boolean {
  try {
    return document.queryCommandState(command);
  } catch {
    return false;
  }
}

/**
 * Paragraphes visés par une commande de bloc.
 *
 * À défaut de sélection connue, la commande porte sur toute la lettre : c'est le seul
 * comportement qui ne perd pas l'intention de l'utilisateur.
 */
function selectedParagraphs(root: HTMLElement, saved: Range | null): HTMLElement[] {
  const children = [...root.children].filter(
    (child): child is HTMLElement => child instanceof HTMLElement,
  );
  const selection = document.getSelection();
  const liveRange =
    selection && selection.rangeCount > 0 && selection.anchorNode
      && root.contains(selection.anchorNode)
      ? selection.getRangeAt(0)
      : saved;
  if (!liveRange) return children;
  const hits = children.filter((child) => {
    try {
      return liveRange.intersectsNode(child);
    } catch {
      return child.contains(liveRange.commonAncestorContainer);
    }
  });
  return hits.length > 0 ? hits : children;
}

/**
 * Lettre en lecture seule, rendue depuis le même modèle que l'éditeur.
 *
 * Le contenu enregistré est du balisage : l'afficher tel quel montrerait les balises, et
 * l'injecter en HTML brut ferait entrer dans la page un contenu qu'on ne contrôle pas. Il
 * est donc relu puis rendu en éléments React.
 */
export function LetterContent({ content }: { content: string }) {
  const paragraphs = parseLetter(content);
  if (paragraphs.length === 0) {
    return (
      <div className="letter-body">
        <p className="text-paper-muted">Lettre vide.</p>
      </div>
    );
  }
  return (
    <div className="letter-body">
      {paragraphs.map((paragraph, index) => (
        <p
          key={index}
          style={paragraph.align === "left" ? undefined : { textAlign: paragraph.align }}
          {...(paragraph.size === "normal" ? {} : { "data-size": paragraph.size })}
        >
          {paragraph.runs.map((run, position) => {
            const content = run.underline ? <u>{run.text}</u> : run.text;
            return run.bold ? (
              <b key={position}>{content}</b>
            ) : (
              <span key={position}>{content}</span>
            );
          })}
        </p>
      ))}
    </div>
  );
}
