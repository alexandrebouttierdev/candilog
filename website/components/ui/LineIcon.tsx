import { cn } from "@/lib/cn";

/**
 * Icônes au trait de Candilog : grille 16, trait 1,4 px, extrémités arrondies,
 * `currentColor`. Les onze premières sont recopiées de l'application
 * (`src/shared/ui/LineIcon.tsx`, `reference_design/assets/icons/`) ; les suivantes sont
 * propres au site et dessinées sur la même grille, au même trait.
 *
 * Pas de police d'icônes (décision D7 de l'application) : un tracé inline hérite de la
 * couleur de son conteneur, donc du thème.
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
  "export-csv": [
    "M8 2.6v7.8M5.2 7.6 8 10.4l2.8-2.8",
    "M2.9 10.6v1.6a1 1 0 0 0 1 1h8.2a1 1 0 0 0 1-1v-1.6",
  ],
  import: [
    "M8 10.6V2.8M5.2 5.6 8 2.8l2.8 2.8",
    "M2.9 10.2v2a1 1 0 0 0 1 1h8.2a1 1 0 0 0 1-1v-2",
  ],
  search: ["M7.3 12.1a4.4 4.4 0 1 0 0-8.8 4.4 4.4 0 0 0 0 8.8Z", "M10.5 10.5 13.4 13.4"],
  /* — Propres au site, même grille — */
  "chevron-down": ["M4.5 6.3 8 9.8l3.5-3.5"],
  "arrow-left": ["M13 8H3.4M7 4.2 3.2 8 7 11.8"],
  "arrow-up-right": ["M5 11 11 5M6 5h5v5"],
  mail: ["M2.8 4.2h10.4v7.6H2.8z", "M3 4.6 8 8.6l5-4"],
  scale: ["M8 2.6v10.8M5 13.4h6", "M3.2 4.6h9.6", "M3.2 4.6 1.8 8.4a1.6 1.6 0 0 0 2.8 0L3.2 4.6ZM12.8 4.6l-1.4 3.8a1.6 1.6 0 0 0 2.8 0l-1.4-3.8Z"],
  sun: [
    "M8 10.6a2.6 2.6 0 1 0 0-5.2 2.6 2.6 0 0 0 0 5.2Z",
    "M8 1.8v1.4M8 12.8v1.4M1.8 8h1.4M12.8 8h1.4M3.6 3.6l1 1M11.4 11.4l1 1M3.6 12.4l1-1M11.4 4.6l1-1",
  ],
  moon: ["M13.2 9.6A5.4 5.4 0 0 1 6.4 2.8a5.4 5.4 0 1 0 6.8 6.8Z"],
  menu: ["M2.5 5h11M2.5 11h11"],
  close: ["M4 4l8 8M12 4l-8 8"],
} as const;

export type LineIconName = keyof typeof PATHS;

/** Icône décorative : le libellé voisin (ou l'`aria-label` du contrôle) porte le sens. */
export function LineIcon({
  name,
  size = 15,
  className,
  strokeWidth = 1.4,
}: {
  name: LineIconName;
  size?: number;
  className?: string;
  strokeWidth?: number;
}) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 16 16"
      fill="none"
      stroke="currentColor"
      strokeWidth={strokeWidth}
      strokeLinecap="round"
      strokeLinejoin="round"
      className={cn("block flex-none", className)}
    >
      {PATHS[name].map((d) => (
        <path key={d} d={d} />
      ))}
    </svg>
  );
}
