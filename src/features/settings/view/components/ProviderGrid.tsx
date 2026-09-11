import { cn } from "@/shared/lib/cn";
import { PROVIDERS, getProvider, idProvider, type ProviderOption } from "../../model/providers";
import type { ProviderKind } from "@/shared/types/generated/settings";
import type { ManagedModelPublisher } from "@/shared/types/generated/ai";

export { getProvider };
import { Icon, Tag } from "@/shared/ui";
import logoOllama from "@/assets/providers/ollama.svg";
import logoClaude from "@/assets/providers/claude.svg";
import logoOpenai from "@/assets/providers/openai.svg";
import logoGemini from "@/assets/providers/googlegemini.svg";
import logoMistral from "@/assets/providers/mistralai.svg";
import logoDeepseek from "@/assets/providers/deepseek.svg";
import logoCustom from "@/assets/providers/custom.svg";
import logoLuth from "@/assets/providers/luth.svg";
import logoCandilogLocal from "@/assets/providers/ollamacandilog.png";

const LOGOS: Record<
  Exclude<ProviderOption["id"], "candilog_local">,
  { src: string; mono: boolean }
> = {
  ollama: { src: logoOllama, mono: true },
  claude: { src: logoClaude, mono: false },
  openai: { src: logoOpenai, mono: true },
  gemini: { src: logoGemini, mono: false },
  mistral: { src: logoMistral, mono: false },
  deepseek: { src: logoDeepseek, mono: false },
  custom: { src: logoCustom, mono: true },
};

/** Logos des éditeurs du catalogue Ollama géré — propriété `publisher`, jamais le nom affiché. */
export const MANAGED_PUBLISHER_LOGOS: Record<
  ManagedModelPublisher,
  { src: string; label: string; mono: boolean }
> = {
  liquid: { src: logoLuth, label: "Liquid", mono: false },
  mistral: { src: logoMistral, label: "Mistral", mono: false },
};

export function logoManagedPublisher(publisher: ManagedModelPublisher) {
  return MANAGED_PUBLISHER_LOGOS[publisher];
}

/** Logo d'éditeur pour les modèles du catalogue Ollama géré. */
export function ManagedPublisherLogo({
  publisher,
  className,
}: {
  publisher: ManagedModelPublisher;
  className?: string;
}) {
  const logo = logoManagedPublisher(publisher);
  return (
    <img
      src={logo.src}
      alt=""
      className={cn("size-5 shrink-0 object-contain", logo.mono && "dark:invert", className)}
    />
  );
}

export const LOGO_CANDILOG_LOCAL = { src: logoCandilogLocal, mono: false };

export function providerLogo(id: ProviderOption["id"]) {
  if (id === "candilog_local") return LOGO_CANDILOG_LOCAL;
  return LOGOS[id];
}

export function ProviderGrid({
  value,
  onChange,
  items = PROVIDERS,
}: {
  value: ProviderKind;
  onChange: (id: ProviderOption["id"]) => void;
  items?: readonly ProviderOption[];
}) {
  const actif = idProvider(value);

  return (
    <div
      role="radiogroup"
      aria-label="Fournisseur IA"
      className="grid gap-2 [grid-template-columns:repeat(auto-fit,minmax(112px,1fr))]"
    >
      {items.map((fournisseur) => {
        const selected = fournisseur.id === actif;
        const logo = providerLogo(fournisseur.id);
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
                : "border-control bg-fill hover:border-control-strong hover:bg-fill-hover",
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
            <span className="relative flex size-10 flex-none items-center justify-center rounded-tile bg-surface">
              {logo ? (
                <img
                  src={logo.src}
                  alt=""
                  width={20}
                  height={20}
                  className={cn("size-5", logo.mono && "dark:invert")}
                />
              ) : (
                <Icon name="smart_toy" size={20} className="text-ink-muted" />
              )}
            </span>
            <span
              className={cn(
                "w-full truncate text-center text-label font-mid",
                selected ? "text-accent" : "text-ink-muted",
              )}
            >
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
