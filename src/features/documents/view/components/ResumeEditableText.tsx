import { useEffect, useRef } from "react";
import type { ClipboardEvent, CSSProperties, ElementType, KeyboardEvent } from "react";

type ResumeEditableTag = "div" | "p" | "span" | "h1" | "h3";

/**
 * Champ éditable d'un CV, sans HTML arbitraire : le contenu affiché ou saisi n'est jamais
 * que du texte brut, jamais un balisage qu'un collage extérieur pourrait injecter.
 *
 * Le DOM n'est réinjecté que lorsque `value` change de l'extérieur (annulation, décision
 * ATS) — jamais pendant la frappe, sinon le curseur repartirait au début à chaque
 * caractère. `tag` choisit l'élément sémantique (titre, paragraphe, portion de ligne) sans
 * dupliquer ce composant pour chaque usage du template.
 */
export function ResumeEditableText({
  value,
  label,
  multiline = false,
  editable,
  tag = "div",
  className,
  onChange,
}: {
  value: string;
  label: string;
  multiline?: boolean;
  editable: boolean;
  tag?: ResumeEditableTag;
  className?: string;
  onChange: (value: string) => void;
}) {
  const zone = useRef<HTMLElement | null>(null);
  // Dernière valeur connue de la zone : si elle correspond déjà à `value`, la modification
  // vient de cette zone elle-même et ne doit pas réinjecter le DOM sous la frappe.
  const derniereValeurConnue = useRef(value);

  useEffect(() => {
    const noeud = zone.current;
    if (!noeud || !editable) return;
    if (value === derniereValeurConnue.current && noeud.textContent === value) return;
    noeud.textContent = value;
    derniereValeurConnue.current = value;
  }, [value, editable]);

  const style: CSSProperties | undefined = multiline ? { whiteSpace: "pre-wrap" } : undefined;
  const Tag = tag as ElementType;

  if (!editable) {
    return (
      <Tag className={className} style={style}>
        {value}
      </Tag>
    );
  }

  const synchroniser = () => {
    const noeud = zone.current;
    if (!noeud) return;
    const suivante = noeud.textContent ?? "";
    derniereValeurConnue.current = suivante;
    onChange(suivante);
  };

  return (
    <Tag
      ref={zone}
      role="textbox"
      aria-label={label}
      aria-multiline={multiline}
      contentEditable
      suppressContentEditableWarning
      className={`${className ?? ""} outline-none`.trim()}
      style={style}
      onInput={synchroniser}
      onBlur={synchroniser}
      onKeyDown={(event: KeyboardEvent) => {
        // Un champ mono-ligne (titre, intitulé, période…) n'a pas de second paragraphe où
        // aller : Entrée y déclenche la validation plutôt qu'un retour à la ligne.
        if (!multiline && event.key === "Enter") event.preventDefault();
      }}
      onPaste={(event: ClipboardEvent<HTMLElement>) => {
        // Le contenu remplace le champ plutôt que de s'insérer au curseur : la position du
        // curseur dans un `contentEditable` n'est pas fiable hors d'un vrai navigateur, et
        // ces champs sont assez courts pour qu'un remplacement complet reste prévisible.
        event.preventDefault();
        const texte = event.clipboardData.getData("text/plain");
        const noeud = zone.current;
        if (noeud) noeud.textContent = texte;
        derniereValeurConnue.current = texte;
        onChange(texte);
      }}
    />
  );
}
