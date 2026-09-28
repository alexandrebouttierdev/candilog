import { cn } from "@/shared/lib/cn";

/**
 * Les 11 icônes dessinées pour Candilog (`reference_design/assets/icons/`) : grille 16,
 * trait 1,4 px, extrémités arrondies, `currentColor` — l'icône suit l'encre de son
 * conteneur, donc l'état actif et le thème.
 *
 * Les tracés sont recopiés ici plutôt qu'importés comme fichiers : un SVG en `<img>` ne
 * peut pas hériter de `currentColor`.
 */
const PATHS = {
  today: ["M8 2.4a5.6 5.6 0 1 0 0 11.2A5.6 5.6 0 0 0 8 2.4Z", "M8 5.2V8l2 1.4"],
  applications: ["M3 4.2h10M3 8h10M3 11.8h6"],
  relations: [
    "M6 7.6a2.1 2.1 0 1 0 0-4.2 2.1 2.1 0 0 0 0 4.2Z",
    "M2.4 13.1c0-2 1.6-3.3 3.6-3.3s3.6 1.3 3.6 3.3",
    "M11 4.1a1.9 1.9 0 0 1 0 3.6M12 9.9c1.1.4 1.9 1.5 1.9 2.9",
  ],
  documents: [
    "M9 2.3H5.1a1.2 1.2 0 0 0-1.2 1.2v9a1.2 1.2 0 0 0 1.2 1.2h5.8a1.2 1.2 0 0 0 1.2-1.2V5.3L9 2.3Z",
    "M9 2.4v3h3",
  ],
  ai: [
    "M8 2.2 9.3 6 13 7.3 9.3 8.6 8 12.4 6.7 8.6 3 7.3 6.7 6 8 2.2Z",
    "M12.4 11.3l.5 1.4 1.3.5-1.3.5-.5 1.4-.5-1.4-1.4-.5 1.4-.5.5-1.4Z",
  ],
  profile: [
    "M8 8.1a2.5 2.5 0 1 0 0-5 2.5 2.5 0 0 0 0 5Z",
    "M3.3 13.7c0-2.3 2.1-3.8 4.7-3.8s4.7 1.5 4.7 3.8",
  ],
  settings: [
    "M8 10a2 2 0 1 0 0-4 2 2 0 0 0 0 4Z",
    "M12.7 9.8a1 1 0 0 0 .2 1.1l.1.1a1.2 1.2 0 1 1-1.7 1.7l-.1-.1a1 1 0 0 0-1.7.7v.1a1.2 1.2 0 1 1-2.4 0v-.1a1 1 0 0 0-1.8-.6l-.1.1a1.2 1.2 0 1 1-1.7-1.7l.1-.1a1 1 0 0 0-.7-1.7h-.1a1.2 1.2 0 1 1 0-2.4h.1a1 1 0 0 0 .6-1.8l-.1-.1a1.2 1.2 0 1 1 1.7-1.7l.1.1a1 1 0 0 0 1.7-.7v-.1a1.2 1.2 0 1 1 2.4 0v.1a1 1 0 0 0 1.8.6l.1-.1a1.2 1.2 0 1 1 1.7 1.7l-.1.1a1 1 0 0 0 .7 1.7h.1a1.2 1.2 0 1 1 0 2.4h-.1a1 1 0 0 0-.9.6Z",
  ],
  bookmark: ["M4 2.6h8a.9.9 0 0 1 .9.9v9.4L8 10.4l-4.9 2.5V3.5a.9.9 0 0 1 .9-.9Z"],
  "export-csv": ["M8 2.6v7.8M5.2 7.6 8 10.4l2.8-2.8", "M2.9 10.6v1.6a1 1 0 0 0 1 1h8.2a1 1 0 0 0 1-1v-1.6"],
  import: ["M8 10.6V2.8M5.2 5.6 8 2.8l2.8 2.8", "M2.9 10.2v2a1 1 0 0 0 1 1h8.2a1 1 0 0 0 1-1v-2"],
  search: ["M7.3 12.1a4.4 4.4 0 1 0 0-8.8 4.4 4.4 0 0 0 0 8.8Z", "M10.5 10.5 13.4 13.4"],
} as const;

export type LineIconName = keyof typeof PATHS;

/** Icône au trait, décorative : le libellé voisin (ou `title` du contrôle) porte le sens. */
export function LineIcon({
  name,
  size = 15,
  className,
}: {
  name: LineIconName;
  /** 13 px (liste), 15 px (navigation). */
  size?: number;
  className?: string | undefined;
}) {
  return (
    <svg
      aria-hidden
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.4}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={cn("flex-none", className)}
    >
      {PATHS[name].map((d) => (
        <path key={d} d={d} />
      ))}
    </svg>
  );
}

/**
 * Tuile de marque Candilog : `#5B62F0` fixe dans les deux thèmes (c'est la marque, elle ne
 * suit pas l'accent — `DECISIONS.md` G1), « C » vectorisé en blanc.
 */
export function BrandMark({ size = 19 }: { size?: number }) {
  return (
    <svg aria-hidden width={size} height={size} viewBox="0 0 64 64" fill="none" className="flex-none">
      <rect width="64" height="64" rx="17" className="fill-brand" />
      <path
        className="fill-white"
        d="M41.6 41.9c-2.4 2.6-5.6 3.9-9.5 3.9-4.3 0-7.8-1.4-10.3-4.1-2.5-2.7-3.8-6.4-3.8-10.9 0-4.4 1.3-8 3.9-10.7 2.6-2.7 6-4.1 10.3-4.1 3.8 0 6.9 1.2 9.3 3.7l-3.6 4.2c-1.6-1.7-3.5-2.5-5.7-2.5-2.5 0-4.4.9-5.9 2.6-1.4 1.7-2.2 4-2.2 6.8 0 2.9.7 5.2 2.2 6.9 1.5 1.7 3.5 2.6 6 2.6 2.3 0 4.3-.9 5.9-2.7l3.4 4.4Z"
      />
    </svg>
  );
}
