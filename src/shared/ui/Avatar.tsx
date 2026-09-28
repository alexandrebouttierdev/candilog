import { cn } from "@/shared/lib/cn";

const FONDS = ["bg-av1", "bg-av2", "bg-av3"] as const;

/**
 * Fond d'avatar stable pour une clé (nom d'entreprise, identifiant) : le même objet garde
 * la même couleur d'un écran à l'autre, cyclique sur les trois fonds du design.
 */
export function avatarTone(key: string): (typeof FONDS)[number] {
  let hash = 0;
  for (const char of key) hash = (hash * 31 + char.charCodeAt(0)) | 0;
  return FONDS[Math.abs(hash) % FONDS.length]!;
}

/** Deux initiales en capitales : « Novéa Services » → « NS », « Linaïa » → « LI ». */
export function avatarInitials(name: string): string {
  const words = name.trim().split(/[\s\-–—·]+/).filter(Boolean);
  if (words.length === 0) return "?";
  if (words.length === 1) return words[0]!.slice(0, 2).toUpperCase();
  return (words[0]![0]! + words[1]![0]!).toUpperCase();
}

/**
 * Avatar d'initiales (`DESIGN_SYSTEM.md` §1.7) : **tuile à rayon 8 pour une entreprise,
 * rond pour une personne** — distinction volontaire du design. Décoratif : le nom est
 * toujours écrit à côté.
 */
export function Avatar({
  name,
  kind,
  size = 19,
  toneKey,
  className,
}: {
  name: string;
  kind: "company" | "person";
  /** Côté en pixels : 18–19 (listes, cartes), 24 (relations), 34 (inspecteur). */
  size?: number;
  /** Clé de couleur, par défaut le nom. */
  toneKey?: string;
  className?: string;
}) {
  return (
    <span
      aria-hidden
      style={{ width: size, height: size, fontSize: Math.max(8, Math.round(size * 0.42)) }}
      className={cn(
        "inline-flex flex-none items-center justify-center font-medium text-av-tx select-none",
        kind === "company" ? (size >= 30 ? "rounded-r9" : "rounded-r8") : "rounded-full",
        avatarTone(toneKey ?? name),
        className,
      )}
    >
      {avatarInitials(name)}
    </span>
  );
}
