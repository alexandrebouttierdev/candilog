import { useNavigate } from "react-router-dom";
import { useQuery } from "@tanstack/react-query";
import { useAiOperationStore } from "@/features/ai/viewmodel/ai-operation-store";
import { useAiRailStatusStore } from "@/features/ai/viewmodel/ai-rail-status-store";
import { iaEstConfiguree } from "@/features/settings/model/etatIa";
import { idProvider } from "@/features/settings/model/providers";
import { railIaStatus } from "@/features/settings/model/railIaStatus";
import {
  defFournisseur,
  logoFournisseur,
} from "@/features/settings/view/components/ProviderGrid";
import { settingsService } from "@/features/settings/services/settingsService";
import { LOCAL_AI_KEY } from "@/features/settings/viewmodel/useLocalAiViewModel";
import { SETTINGS_KEY } from "@/features/settings/viewmodel/useSettingsViewModel";
import { useSystemResources } from "@/features/settings/viewmodel/useSystemResources";
import { localAiService } from "@/features/settings/services/localAiService";
import { cn } from "@/shared/lib/cn";
import { Icon } from "@/shared/ui/Icon";
import type { Tone } from "@/shared/ui";

const DOT: Record<Tone, string> = {
  success: "bg-success",
  warning: "bg-warning",
  danger: "bg-danger",
  neutral: "bg-ink-subtle",
  accent: "bg-accent",
};

function formatPercent(value: number | undefined): string {
  if (value === undefined || !Number.isFinite(value)) return "—";
  return `${Math.round(value)} %`;
}

function Meter({
  label,
  percent,
  available,
}: {
  label: string;
  percent: number | undefined;
  available: boolean;
}) {
  const width = available && percent !== undefined ? Math.min(100, Math.max(0, percent)) : 0;
  return (
    <div className="flex w-full flex-col gap-0.5 px-1" title={`${label} ${formatPercent(percent)}`}>
      <div className="flex items-baseline justify-between gap-1">
        <span className="text-micro font-medium text-ink-subtle">{label}</span>
        <span className="font-mono text-micro text-ink-muted">
          {available ? formatPercent(percent) : "—"}
        </span>
      </div>
      <div
        className="h-1 w-full overflow-hidden rounded-chip bg-fill"
        role="progressbar"
        aria-label={label}
        aria-valuemin={0}
        aria-valuemax={100}
        aria-valuenow={available && percent !== undefined ? Math.round(percent) : undefined}
        aria-valuetext={available ? formatPercent(percent) : "indisponible"}
      >
        <div
          className={cn("h-full rounded-chip", available ? "bg-accent" : "bg-fill")}
          style={{ width: `${width}%` }}
        />
      </div>
    </div>
  );
}

/** Indicateur fournisseur IA + charge machine, compact dans le rail 68 px. */
export function AiProviderRailWidget() {
  const navigate = useNavigate();
  const settings = useQuery({
    queryKey: SETTINGS_KEY,
    queryFn: settingsService.load,
  });
  const llm = settings.data?.llm;
  const configured = llm ? iaEstConfiguree(llm) : false;
  const providerId = llm ? idProvider(llm.provider) : null;
  const fournisseur = llm ? defFournisseur(llm.provider) : null;
  const logo = providerId && configured ? logoFournisseur(providerId) : null;

  const busy = useAiOperationStore((state) => state.active !== null);
  const connectionTest = useAiRailStatusStore((state) => state.connectionTest);
  const operationError = useAiRailStatusStore((state) => state.lastOperationFailed);

  const localQuery = useQuery({
    queryKey: LOCAL_AI_KEY,
    queryFn: async () => {
      const [recommendation, status] = await Promise.all([
        localAiService.recommendation(),
        localAiService.status(),
      ]);
      return { recommendation, status };
    },
    enabled: providerId === "mistral_local",
  });
  const localError =
    providerId === "mistral_local" &&
    (localQuery.data?.status.state === "error" || Boolean(localQuery.data?.status.last_error));

  const status = railIaStatus({
    configured,
    busy,
    localError: Boolean(localError),
    connectionError: connectionTest === "error",
    operationError,
  });

  const resources = useSystemResources();
  const snap = resources.data;

  const nom = configured && fournisseur ? fournisseur.label : "Aucun fournisseur";
  const tooltip = `${nom} — ${status.label}`;

  return (
    <div className="mb-1.5 flex w-full flex-col items-center gap-1.5 px-1">
      <div className="group relative flex justify-center">
        <button
          type="button"
          title={tooltip}
          aria-label={tooltip}
          onClick={() => navigate("/settings/ai")}
          className={cn(
            "relative flex h-9 w-[42px] flex-none items-center justify-center rounded-tile",
            "text-ink-subtle transition-colors duration-hover hover:bg-surface-hover hover:text-ink-muted",
            "focus-visible:outline-1 focus-visible:outline-accent-focus",
          )}
        >
          {logo ? (
            <img
              src={logo.src}
              alt=""
              width={20}
              height={20}
              className={cn("size-5 object-contain", logo.mono && "dark:invert")}
            />
          ) : (
            <Icon name="smart_toy" size={20} />
          )}
          <span
            aria-hidden
            className={cn(
              "absolute right-1 bottom-1 size-2 rounded-full ring-2 ring-page",
              DOT[status.tone],
            )}
          />
        </button>
        <span className="pointer-events-none absolute top-1/2 left-[54px] z-[60] flex -translate-y-1/2 items-center gap-2.5 rounded-button border border-overlay bg-[var(--candilog-glass-menu)] px-2.5 py-1.5 whitespace-nowrap opacity-0 shadow-menu backdrop-blur-[14px] group-hover:opacity-100">
          <span className="text-note whitespace-nowrap text-ink">{tooltip}</span>
        </span>
      </div>
      <div className="flex w-full flex-col gap-1">
        <Meter label="CPU" percent={snap?.cpu_percent} available={snap !== undefined} />
        <Meter label="RAM" percent={snap?.ram_used_percent} available={snap !== undefined} />
        <Meter
          label="VRAM"
          percent={snap?.vram_used_percent ?? undefined}
          available={Boolean(snap?.vram_available)}
        />
      </div>
    </div>
  );
}
