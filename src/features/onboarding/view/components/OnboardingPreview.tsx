import { cn } from "@/shared/lib/cn";
import { Icon } from "@/shared/ui";
import logoCandilog from "@/assets/logo-candilog.svg";
import logoCandilogDark from "@/assets/logo-candilog-dark.svg";
import logoOllama from "@/assets/providers/ollama.svg";
import logoClaude from "@/assets/providers/claude.svg";
import logoOpenai from "@/assets/providers/openai.svg";
import logoGemini from "@/assets/providers/googlegemini.svg";
import { useUiStore } from "@/shared/lib/ui-store";
import type { OnboardingPreviewKind } from "../../model/steps";

/**
 * Frame commun : chrome minimal, puis une miniature du vrai écran.
 *
 * Le contenu reprend les libellés réels de l'application — « En attente », « Taux de
 * réponse », « Ollama » — plutôt que des barres grises : une maquette abstraite se lit
 * comme un squelette de chargement, pas comme un aperçu.
 */
function Frame({ children }: { children: React.ReactNode }) {
  return (
    <div className="h-[184px] w-full overflow-hidden rounded-card border border-line bg-surface shadow-e1">
      <div className="flex h-7 flex-none items-center gap-1.5 border-b border-line px-3">
        <span className="size-1.5 rounded-full bg-neutral-tint" />
        <span className="size-1.5 rounded-full bg-neutral-tint" />
        <span className="size-1.5 rounded-full bg-neutral-tint" />
      </div>
      <div className="flex h-[calc(100%-28px)] flex-col gap-2 bg-page p-2.5">{children}</div>
    </div>
  );
}

/** Pastille d'initiales, comme les listes Relations. */
function Initials({ children }: { children: string }) {
  return (
    <span className="flex size-5 flex-none items-center justify-center rounded-full bg-accent-tint text-tag font-strong text-accent">
      {children}
    </span>
  );
}

function TodayPreview() {
  const stats = [
    { label: "Candidatures", value: "46" },
    { label: "Réponses", value: "14" },
    { label: "Entretiens", value: "2" },
  ];
  return (
    <Frame>
      <div className="flex gap-2">
        {stats.map((stat) => (
          <div key={stat.label} className="flex-1 rounded-tile border border-line bg-surface px-2 py-1.5">
            <p className="truncate text-tag uppercase text-ink-label">{stat.label}</p>
            <p className="tabular font-mono text-item text-ink">{stat.value}</p>
          </div>
        ))}
      </div>
      <div className="flex-1 rounded-tile border border-line bg-surface p-2">
        <p className="mb-1.5 text-tag uppercase tracking-[0.07em] text-ink-label">Prochainement</p>
        <div className="flex items-center gap-2">
          <span className="tabular font-mono text-tag text-accent">Auj.</span>
          <Initials>LX</Initials>
          <span className="min-w-0 flex-1 truncate text-tag text-ink">Linxea</span>
          <span className="text-tag text-warning">Relance</span>
        </div>
        <div className="mt-1.5 flex items-center gap-2 border-t border-field pt-1.5">
          <span className="tabular font-mono text-tag text-ink-faint">Jeu.</span>
          <Initials>ND</Initials>
          <span className="min-w-0 flex-1 truncate text-tag text-ink">Nova Digital</span>
          <span className="text-tag text-accent-text">Entretien</span>
        </div>
      </div>
    </Frame>
  );
}

function KanbanPreview() {
  const columns = [
    { label: "En attente", count: "12", cards: ["Nova Digital", "ISCOD"] },
    { label: "Relancée", count: "5", cards: ["Linxea"] },
    { label: "Entretien", count: "2", cards: ["Atlas Studio"], actif: true },
    { label: "Refus", count: "9", cards: [] },
  ];
  return (
    <Frame>
      <div className="flex h-full gap-1.5">
        {columns.map((column) => (
          <div key={column.label} className="flex flex-1 flex-col gap-1 rounded-tile border border-line bg-surface p-1.5">
            <div className="flex items-center gap-1">
              <span className="min-w-0 truncate text-tag font-mid text-ink">{column.label}</span>
              <span className="tabular rounded-chip bg-fill px-1 text-tag text-ink-muted">{column.count}</span>
            </div>
            {column.cards.map((card) => (
              <span
                key={card}
                className={cn(
                  "block truncate rounded-[5px] border px-1 py-1 text-tag",
                  column.actif
                    ? "border-accent-border bg-accent-tint text-accent"
                    : "border-line bg-fill text-ink-muted",
                )}
              >
                {card}
              </span>
            ))}
          </div>
        ))}
      </div>
    </Frame>
  );
}

function NetworkPreview() {
  const rows = [
    { initials: "ND", name: "Nova Digital", detail: "ESN · Rennes" },
    { initials: "AS", name: "Atlas Studio", detail: "Client final · Nantes" },
    { initials: "CR", name: "Camille Rivet", detail: "Recruteuse" },
  ];
  return (
    <Frame>
      {rows.map((row) => (
        <div key={row.name} className="flex items-center gap-2 rounded-tile border border-line bg-surface px-2 py-1.5">
          <Initials>{row.initials}</Initials>
          <div className="min-w-0 flex-1">
            <p className="truncate text-tag font-mid text-ink">{row.name}</p>
            <p className="truncate text-tag text-ink-faint">{row.detail}</p>
          </div>
        </div>
      ))}
    </Frame>
  );
}

function DocumentsPreview() {
  return (
    <Frame>
      <div className="flex h-full gap-2">
        <div className="flex flex-1 flex-col gap-1 rounded-tile border border-line bg-surface p-2">
          <span className="mb-0.5 inline-flex w-fit items-center gap-1 rounded-chip bg-accent-tint px-1 py-0.5 text-tag font-mid text-accent">
            <Icon name="auto_awesome" size={9} />
            CV ciblé
          </span>
          <p className="truncate text-tag font-strong text-ink">Camille Rivet</p>
          <p className="truncate text-tag text-ink-faint">Développeuse Front-end</p>
          <span className="mt-0.5 block h-px bg-line" />
          <span className="block h-1 w-full rounded-full bg-fill" />
          <span className="block h-1 w-4/5 rounded-full bg-fill" />
          <span className="block h-1 w-3/5 rounded-full bg-fill" />
        </div>
        <div className="flex flex-1 flex-col gap-1 rounded-tile border border-line bg-surface p-2">
          <p className="truncate text-tag font-strong text-ink">Lettre de motivation</p>
          <p className="truncate text-tag text-ink-faint">Nova Digital</p>
          <span className="mt-0.5 block h-px bg-line" />
          <span className="block h-1 w-full rounded-full bg-fill" />
          <span className="block h-1 w-full rounded-full bg-fill" />
          <span className="block h-1 w-2/3 rounded-full bg-fill" />
          <span className="tabular mt-auto text-tag text-success">Score ATS 82</span>
        </div>
      </div>
    </Frame>
  );
}

function AnalyticsPreview() {
  const semaines = [
    { label: "S31", height: 45 },
    { label: "S32", height: 75 },
    { label: "S33", height: 40 },
    { label: "S34", height: 100, actif: true },
    { label: "S35", height: 60 },
  ];
  return (
    <Frame>
      <div className="flex items-baseline justify-between">
        <p className="text-tag uppercase tracking-[0.07em] text-ink-label">Taux de réponse</p>
        <p className="tabular font-mono text-item text-accent">30 %</p>
      </div>
      {/* Hauteurs en pixels, pas en pourcentage : le conteneur n'a pas de hauteur définie,
          un `height: 45%` s'y résoudrait à zéro et les barres disparaîtraient. */}
      <div className="flex flex-1 items-end gap-1.5">
        {semaines.map((semaine) => (
          <div key={semaine.label} className="flex flex-1 flex-col items-center gap-1">
            <span
              style={{ height: `${Math.round(semaine.height * 0.62)}px` }}
              className={cn("w-full rounded-t-[3px]", semaine.actif ? "bg-accent" : "bg-accent-tint")}
            />
            <span className="text-tag text-ink-faint">{semaine.label}</span>
          </div>
        ))}
      </div>
    </Frame>
  );
}

function ProfilePreview() {
  return (
    <Frame>
      <div className="flex items-center gap-2">
        <span className="flex size-8 flex-none items-center justify-center rounded-full bg-accent-tint text-tag font-strong text-accent">
          CR
        </span>
        <div className="min-w-0 flex-1">
          <p className="truncate text-item text-ink">Camille Rivet</p>
          <p className="truncate text-tag text-accent">Développeuse Front-end</p>
        </div>
      </div>
      <div>
        <div className="mb-1 flex items-baseline justify-between">
          <span className="text-tag text-ink-muted">Profil complété</span>
          <span className="tabular text-tag font-strong text-accent">62 %</span>
        </div>
        <div className="h-1.5 w-full overflow-hidden rounded-full bg-neutral-tint">
          <div className="h-full w-[62%] rounded-full bg-accent" />
        </div>
      </div>
      <div className="flex flex-1 items-start gap-1.5">
        {["Expériences 3", "Compétences 8", "Formations 2"].map((onglet) => (
          <span
            key={onglet}
            className="flex-1 truncate rounded-tile border border-line bg-surface px-1.5 py-1 text-center text-tag text-ink-muted"
          >
            {onglet}
          </span>
        ))}
      </div>
    </Frame>
  );
}

function AiPreview() {
  const providers = [
    { label: "Ollama", logo: logoOllama, mono: true, local: true },
    { label: "Claude", logo: logoClaude, mono: false, local: false },
    { label: "OpenAI", logo: logoOpenai, mono: true, local: false },
    { label: "Gemini", logo: logoGemini, mono: false, local: false },
  ];
  return (
    <Frame>
      <div className="grid grid-cols-4 gap-1.5">
        {providers.map((provider) => (
          <div
            key={provider.label}
            className={cn(
              "flex flex-col items-center gap-1 rounded-tile border p-1.5",
              provider.local ? "border-accent-border bg-accent-tint" : "border-line bg-surface",
            )}
          >
            <img
              src={provider.logo}
              alt=""
              width={14}
              height={14}
              className={cn("size-3.5", provider.mono && "dark:invert")}
            />
            <span
              className={cn(
                "w-full truncate text-center text-tag",
                provider.local ? "text-accent" : "text-ink-muted",
              )}
            >
              {provider.label}
            </span>
            {provider.local ? (
              <span className="rounded-chip bg-neutral-tint px-1 text-tag text-ink-muted">Local</span>
            ) : null}
          </div>
        ))}
      </div>
      <div className="flex-1 rounded-tile border border-line bg-surface p-2">
        <p className="mb-1 text-tag uppercase tracking-[0.07em] text-ink-label">Modèle</p>
        <p className="tabular truncate font-mono text-tag text-ink">llama3.2:3b</p>
      </div>
    </Frame>
  );
}

function BrandMark() {
  const theme = useUiStore((state) => state.theme);
  const sombre =
    theme === "dark" ||
    (theme === "system" &&
      typeof window !== "undefined" &&
      typeof window.matchMedia === "function" &&
      window.matchMedia("(prefers-color-scheme: dark)").matches);
  return (
    <div className="flex h-[184px] w-full flex-col items-center justify-center gap-3 rounded-card border border-line bg-surface shadow-e1">
      <img src={sombre ? logoCandilogDark : logoCandilog} alt="" width={56} height={56} />
      <p className="text-note text-ink-faint">Tout reste sur votre machine</p>
    </div>
  );
}

const PREVIEWS: Record<OnboardingPreviewKind, () => React.JSX.Element> = {
  welcome: BrandMark,
  today: TodayPreview,
  kanban: KanbanPreview,
  network: NetworkPreview,
  documents: DocumentsPreview,
  analytics: AnalyticsPreview,
  profile: ProfilePreview,
  ai: AiPreview,
  closing: BrandMark,
};

/**
 * Aperçu de l'écran présenté par une étape du tour.
 *
 * Une miniature dessinée avec les jetons du design system plutôt qu'une capture d'écran :
 * elle reste lisible en clair comme en sombre sans double jeu d'images à maintenir, et ne
 * date pas au premier écran retouché. Décorative : le texte de l'étape dit déjà la même
 * chose, la répéter au lecteur d'écran n'apporterait rien.
 */
export function OnboardingPreview({ kind }: { kind: OnboardingPreviewKind }) {
  const Preview = PREVIEWS[kind];
  return (
    <div aria-hidden="true">
      <Preview />
    </div>
  );
}
