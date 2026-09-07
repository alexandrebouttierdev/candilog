import { cn } from "@/shared/lib/cn";
import {
  FOURNISSEURS,
  idProvider,
  type FournisseurOption,
} from "../../model/providers";
import type { ProviderKind } from "@/shared/types/generated/settings";
import type { LocalModelFamily } from "@/shared/types/generated/ai";
import { Icon, Tag } from "@/shared/ui";
import logoOllama from "@/assets/providers/ollama.svg";
import logoClaude from "@/assets/providers/claude.svg";
import logoOpenai from "@/assets/providers/openai.svg";
import logoGemini from "@/assets/providers/googlegemini.svg";
import logoMistral from "@/assets/providers/mistralai.svg";
import logoLuth from "@/assets/providers/luth.svg";
import logoDeepseek from "@/assets/providers/deepseek.svg";
import logoCustom from "@/assets/providers/custom.svg";

const LOGOS: Record<Exclude<FournisseurOption["id"], "mistral_local">, { src: string; mono: boolean }> = {
  ollama: { src: logoOllama, mono: true },
  claude: { src: logoClaude, mono: false },
  openai: { src: logoOpenai, mono: true },
  gemini: { src: logoGemini, mono: false },
  mistral: { src: logoMistral, mono: false },
  deepseek: { src: logoDeepseek, mono: false },
  custom: { src: logoCustom, mono: true },
};

/** Logos des familles d'artefacts locaux — propriété `family`, jamais le nom affiché. */
export const LOCAL_FAMILY_LOGOS: Record<
  LocalModelFamily,
  { src: string; label: string; mono: boolean }
> = {
  mistral: { src: logoMistral, label: "Mistral", mono: false },
  luth: { src: logoLuth, label: "Luth", mono: false },
};

export function logoFournisseur(id: FournisseurOption["id"]) {
  if (id === "mistral_local") return null;
  return LOGOS[id];
}

/**
 * Logo à afficher pour l'IA locale : famille active si connue, sinon générique (`null`
 * → `smart_toy` côté UI). Pas de pile Mistral+Luth.
 */
export function logoIaLocale(family: LocalModelFamily | null | undefined) {
  return family ? LOCAL_FAMILY_LOGOS[family] : null;
}

export function defFournisseur(provider: ProviderKind): FournisseurOption {
  const id = idProvider(provider);
  return FOURNISSEURS.find((item) => item.id === id) ?? FOURNISSEURS[0]!;
}

/**
 * Tuiles de fournisseur : logo, nom, et sélection portée par la tuile elle-même.
 *
 * Une tuile bordée dit qu'elle se clique ; l'ancienne grille sans filet laissait sept logos
 * de 18 px flotter sur toute la largeur et ne se distinguait d'une légende que par le
 * curseur.
 *
 * La sélection ne peut pas se contenter du couple `accent-border` / `accent-tint` employé
 * par les listes : celles-ci laissent leurs items non choisis en `border-transparent`, si
 * bien que le filet accent surgit du néant. Ici les huit tuiles sont déjà bordées et
 * remplies — un filet à 28 % d'opacité (22 % en sombre) et un fond à 10 % ne changeaient
 * que la teinte, à valeur presque constante, et le choix se devinait à peine. La tuile
 * choisie porte donc un filet accent **plein**, la teinte haute, et une pastille de
 * validation : un repère qui survit aux deux thèmes et à une vision des couleurs atypique.
 */
export function ProviderGrid({
  value,
  onChange,
}: {
  value: ProviderKind;
  onChange: (id: FournisseurOption["id"]) => void;
}) {
  const actif = idProvider(value);

  return (
    <div
      role="radiogroup"
      aria-label="Fournisseur IA"
      className="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(112px,1fr))]"
    >
      {FOURNISSEURS.map((fournisseur) => {
        const selected = fournisseur.id === actif;
        const logo =
          fournisseur.id === "mistral_local" ? null : LOGOS[fournisseur.id];
        return (
          <button
            key={fournisseur.id}
            type="button"
            role="radio"
            aria-checked={selected}
            aria-label={fournisseur.label}
            onClick={() => onChange(fournisseur.id)}
            className={cn(
              "relative flex min-w-0 flex-col items-center gap-2 rounded-tile border px-2 py-3",
              "transition-[background-color,border-color] duration-hover ease-in-out",
              "focus-visible:outline-1 focus-visible:outline-accent-focus",
              selected
                ? "border-accent bg-accent-tint-12"
                : "border-control bg-fill hover:bg-fill-hover",
            )}
          >
            {selected ? (
              <Icon
                name="check_circle"
                size={16}
                filled
                className="absolute right-1.5 top-1.5 text-accent"
              />
            ) : null}
            <span
              className="relative flex size-9 flex-none items-center justify-center rounded-control bg-surface"
            >
              {logo ? (
                <img
                  src={logo.src}
                  alt=""
                  width={20}
                  height={20}
                  className={cn("size-5", logo.mono && "dark:invert")}
                />
              ) : (
                // IA locale : icône générique tant qu'aucune famille n'est active ailleurs.
                <Icon name="smart_toy" size={20} className="text-ink-muted" />
              )}
            </span>
            <span className={cn("w-full truncate text-center text-label font-mid", selected ? "text-accent" : "text-ink-muted")}>
              {fournisseur.label}
            </span>
            <span className="min-h-8 text-center text-meta leading-tight text-ink-faint">
              {fournisseur.hint}
            </span>
            {fournisseur.recommended ? <Tag>Recommandé</Tag> : null}
          </button>
        );
      })}
    </div>
  );
}
