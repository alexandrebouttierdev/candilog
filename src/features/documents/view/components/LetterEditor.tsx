import { useEffect, useRef, useState } from "react";
import { IconButton, Select } from "@/shared/ui";
import { DocumentPanel } from "./DocumentUi";
import { LetterPaper, type LetterPaperField, type LetterPaperFields } from "./LetterPaper";
import {
  markupFromDom,
  parseLetter,
  toEditableHtml,
  type LetterAlign,
  type LetterSize,
} from "../../model/letterMarkup";

const ALIGNEMENTS: { value: LetterAlign; icon: string; label: string }[] = [
  { value: "left", icon: "format_align_left", label: "Aligner à gauche" },
  { value: "center", icon: "format_align_center", label: "Centrer" },
  { value: "right", icon: "format_align_right", label: "Aligner à droite" },
];

/**
 * Éditeur de la lettre.
 *
 * La barre d'outils vit dans l'en-tête du panneau, **hors de la feuille** : posée sur le
 * papier elle défilait avec le texte et prenait la place de la lettre. Elle ne propose que
 * ce que l'export PDF sait honorer — gras, souligné, taille et alignement — parce qu'un
 * bouton dont l'effet disparaît à l'impression est un piège.
 *
 * Le contenu vit dans le DOM de la zone éditable et n'est relu qu'à la frappe : réinjecter
 * le balisage à chaque rendu replacerait le curseur au début à chaque lettre tapée.
 */
export function LetterEditor({
  value,
  readOnly,
  fields,
  onChange,
  onFieldsChange,
  onOverflowChange,
}: {
  value: string;
  readOnly: boolean;
  fields: LetterPaperFields;
  onChange: (markup: string) => void;
  onFieldsChange: (field: LetterPaperField, value: string) => void;
  onOverflowChange: (overflow: boolean) => void;
}) {
  const zone = useRef<HTMLDivElement | null>(null);
  const dernier = useRef<string>("");
  // Dernière sélection connue **dans la lettre** : ouvrir la liste des tailles déplace le
  // curseur hors de la zone, et sans ce repère la commande ne saurait plus sur quoi agir.
  const plage = useRef<Range | null>(null);
  const [actif, setActif] = useState({ bold: false, underline: false });

  useEffect(() => {
    const racine = zone.current;
    if (!racine || value === dernier.current) return;
    racine.innerHTML = toEditableHtml(parseLetter(value));
    dernier.current = value;
  }, [value]);

  useEffect(() => {
    const rafraichir = () => {
      const racine = zone.current;
      const selection = document.getSelection();
      if (!racine || !selection?.anchorNode || !racine.contains(selection.anchorNode)) return;
      if (selection.rangeCount > 0) plage.current = selection.getRangeAt(0).cloneRange();
      setActif({ bold: etatCommande("bold"), underline: etatCommande("underline") });
    };
    document.addEventListener("selectionchange", rafraichir);
    return () => document.removeEventListener("selectionchange", rafraichir);
  }, []);

  const synchroniser = () => {
    const racine = zone.current;
    if (!racine) return;
    const markup = markupFromDom(racine);
    dernier.current = markup;
    onChange(markup);
  };

  const appliquer = (commande: "bold" | "underline") => {
    reprendreLaMain();
    executer(commande);
    setActif({ bold: etatCommande("bold"), underline: etatCommande("underline") });
    synchroniser();
  };

  const surLesParagraphes = (action: (element: HTMLElement) => void) => {
    const racine = zone.current;
    if (!racine) return;
    reprendreLaMain();
    for (const cible of paragraphesSelectionnes(racine, plage.current)) action(cible);
    synchroniser();
  };

  /** Redonne le curseur à la lettre, là où il était avant le clic sur la barre d'outils. */
  const reprendreLaMain = () => {
    const racine = zone.current;
    if (!racine) return;
    racine.focus();
    const memorisee = plage.current;
    if (!memorisee || !racine.contains(memorisee.commonAncestorContainer)) return;
    const selection = document.getSelection();
    selection?.removeAllRanges();
    selection?.addRange(memorisee);
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
        aria-pressed={actif.bold}
        disabled={readOnly}
        className={actif.bold ? "bg-accent-tint text-accent" : undefined}
        onClick={() => appliquer("bold")}
      />
      <IconButton
        icon="format_underlined"
        label="Souligné"
        aria-pressed={actif.underline}
        disabled={readOnly}
        className={actif.underline ? "bg-accent-tint text-accent" : undefined}
        onClick={() => appliquer("underline")}
      />
      <span aria-hidden className="mx-1 h-4 w-px bg-line" />
      {ALIGNEMENTS.map((alignement) => (
        <IconButton
          key={alignement.value}
          icon={alignement.icon}
          label={alignement.label}
          disabled={readOnly}
          onClick={() =>
            surLesParagraphes((element) => {
              element.style.textAlign = alignement.value === "left" ? "" : alignement.value;
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
          const taille = event.target.value as LetterSize;
          surLesParagraphes((element) => {
            if (taille === "normal") delete element.dataset["size"];
            else element.dataset["size"] = taille;
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
          editable={!readOnly}
          onChange={onFieldsChange}
          onOverflowChange={onOverflowChange}
        >
          <div
            ref={zone}
            role="textbox"
            aria-multiline="true"
            aria-label="Contenu de la lettre"
            contentEditable={!readOnly}
            suppressContentEditableWarning
            data-placeholder="La lettre apparaîtra ici après la rédaction. Vous pouvez aussi l'écrire directement."
            onInput={synchroniser}
            onBlur={synchroniser}
            onPaste={(event) => {
              // Un collage extérieur arrive avec ses propres styles : seul son texte entre.
              event.preventDefault();
              executer("insertText", event.clipboardData.getData("text/plain"));
              synchroniser();
            }}
            className="letter-body outline-none"
          />
        </LetterPaper>
      </div>
    </DocumentPanel>
  );
}

/** `execCommand` reste le seul moyen d'éditer une sélection sans embarquer un éditeur tiers. */
function executer(commande: string, valeur?: string): void {
  try {
    document.execCommand(commande, false, valeur);
  } catch {
    // Moteur sans support : la frappe directe reste possible, la mise en forme non.
  }
}

function etatCommande(commande: string): boolean {
  try {
    return document.queryCommandState(commande);
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
function paragraphesSelectionnes(racine: HTMLElement, memorisee: Range | null): HTMLElement[] {
  const enfants = [...racine.children].filter(
    (enfant): enfant is HTMLElement => enfant instanceof HTMLElement,
  );
  const selection = document.getSelection();
  const vivante =
    selection && selection.rangeCount > 0 && selection.anchorNode
      && racine.contains(selection.anchorNode)
      ? selection.getRangeAt(0)
      : memorisee;
  if (!vivante) return enfants;
  const touches = enfants.filter((enfant) => {
    try {
      return vivante.intersectsNode(enfant);
    } catch {
      return enfant.contains(vivante.commonAncestorContainer);
    }
  });
  return touches.length > 0 ? touches : enfants;
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
            const texte = run.underline ? <u>{run.text}</u> : run.text;
            return run.bold ? <b key={position}>{texte}</b> : <span key={position}>{texte}</span>;
          })}
        </p>
      ))}
    </div>
  );
}
