import { cn } from "@/shared/lib/cn";
import { PROVIDERS, getProvider, idProvider, type ProviderOption } from "../../model/providers";
import type { ProviderKind } from "@/shared/types/generated/settings";
import type { ManagedModelPublisher } from "@/shared/types/generated/ai";

export { getProvider };
import { LineIcon, Tag } from "@/shared/ui";
import logoOllama from "@/assets/providers/ollama.svg";
import logoClaude from "@/assets/providers/claude.svg";
import logoOpenai from "@/assets/providers/openai.svg";
import logoGemini from "@/assets/providers/googlegemini.svg";
import logoMistral from "@/assets/providers/mistralai.svg";
import logoDeepseek from "@/assets/providers/deepseek.svg";
import logoCustom from "@/assets/providers/custom.svg";
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

/**
 * Logos des éditeurs du catalogue Ollama géré — propriété `publisher`, jamais le nom affiché.
 *
 * Volontairement **partiel** : tant qu'on ne dispose pas de la marque officielle d'un
 * éditeur, mieux vaut n'afficher aucun logo que d'en détourner un autre. Gemma et Gemini
 * sont deux produits Google distincts, et `googlegemini.svg` — qui porte bien le titre
 * « Google Gemini » — reste réservé au fournisseur distant Gemini, où il est juste.
 */
export const MANAGED_PUBLISHER_LOGOS: Partial<
  Record<ManagedModelPublisher, { src: string; label: string; mono: boolean }>
> = {
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
  if (!logo) {
    // Repli sur l'initiale de l'éditeur, comme `Avatar` : la tuile reste occupée et
    // l'identité lisible, sans inventer une marque que nous n'avons pas.
    return (
      <span className={cn("text-small font-medium text-tx-4", className)}>
        {publisherInitial(publisher)}
      </span>
    );
  }
  return (
    <img
      src={logo.src}
      alt=""
      className={cn("size-5 shrink-0 object-contain", logo.mono && "dark:invert", className)}
    />
  );
}

/** Initiale d'un éditeur sans logo officiel disponible. */
function publisherInitial(publisher: ManagedModelPublisher): string {
  return publisher.charAt(0).toUpperCase();
}

/**
 * Logo d'un fournisseur, pour les surfaces qui le nomment hors de la grille — onglets
 * verticaux de l'écran IA, sélecteur de modèles distants.
 *
 * Composant et non `<img>` recopié : la bascule `mono` (inversion en thème sombre des
 * marques monochromes — OpenAI, Ollama, Personnalisé) vivait déjà en deux endroits, et une
 * troisième copie aurait fini par diverger.
 */
export function ProviderLogo({
  id,
  className,
}: {
  id: ProviderOption["id"];
  className?: string;
}) {
  const logo = providerLogo(id);
  if (!logo) return null;
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
              <span aria-hidden className="absolute right-2 top-1 text-accent">
                ✓
              </span>
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
                <LineIcon name="ai" size={18} className="text-ink-muted" />
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
