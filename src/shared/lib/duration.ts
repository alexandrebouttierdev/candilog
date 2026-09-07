/** Horloge mm:ss, ou hh:mm:ss au-delà d'une heure. */
export function formatElapsed(ms: number): string {
  const total = Math.max(0, Math.floor(ms / 1000));
  const hours = Math.floor(total / 3600);
  const minutes = Math.floor((total % 3600) / 60);
  const seconds = total % 60;
  const pad = (value: number) => String(value).padStart(2, "0");
  if (hours > 0) {
    return `${pad(hours)}:${pad(minutes)}:${pad(seconds)}`;
  }
  return `${pad(minutes)}:${pad(seconds)}`;
}

/** Durée totale affichée à la fin, virgule française. */
export function formatDuration(ms: number): string {
  const seconds = Math.max(0, ms / 1000);
  if (seconds >= 60) {
    const minutes = Math.floor(seconds / 60);
    const rest = Math.round(seconds % 60);
    return rest === 0 ? `${minutes} min` : `${minutes} min ${rest} s`;
  }
  const rounded = Number.isInteger(seconds) ? seconds.toFixed(0) : seconds.toFixed(1);
  return `${rounded.replace(".", ",")} s`;
}

/**
 * Nombre de tokens, séparateur de milliers français.
 *
 * Affiché à côté du temps écoulé dans `AiProgress` : les deux mesures répondent à la même
 * question pendant un traitement IA — combien ça coûte, en temps et en tokens.
 */
export function formatTokens(count: number): string {
  return new Intl.NumberFormat("fr-FR").format(count);
}

/** Récapitulatif conservé après un traitement IA terminé. */
export function formatAiSummary(
  action: string,
  elapsedMs: number,
  tokensUsed: number | null,
): string {
  const tokens =
    tokensUsed === null
      ? "tokens non communiqués"
      : `${formatTokens(tokensUsed)} tokens`;
  return `${action} en ${formatDuration(elapsedMs)} · ${tokens}`;
}


/** Débit tokens/s, une décimale, virgule française. */
export function formatTokensPerSecond(rate: number): string {
  return `${rate.toLocaleString("fr-FR", {
    minimumFractionDigits: 1,
    maximumFractionDigits: 1,
  })} tokens/s`;
}

/**
 * Débit affiché pendant une progression IA.
 *
 * Préfère la mesure native (`tokens_per_second`) quand le runtime local la publie ;
 * sinon estime `tokens / elapsed` dès qu'un total de tokens et au moins une seconde
 * sont connus — typiquement un fournisseur distant après le premier appel terminé.
 */
export function resolveTokensPerSecond(
  tokensUsed: number | null | undefined,
  elapsedMs: number,
  explicit: number | null | undefined = null,
): number | null {
  if (explicit !== null && explicit !== undefined && Number.isFinite(explicit) && explicit > 0) {
    return explicit;
  }
  if (tokensUsed === null || tokensUsed === undefined || tokensUsed <= 0 || elapsedMs < 1_000) {
    return null;
  }
  return tokensUsed / (elapsedMs / 1_000);
}

/** Ligne de métriques : temps, tokens, débit éventuel. */
export function formatProgressMetrics(
  elapsedMs: number,
  tokensUsed: number | null | undefined,
  tokensPerSecond: number | null | undefined = null,
): string {
  const parts = [`Temps écoulé : ${formatElapsed(elapsedMs)}`];
  if (tokensUsed !== null && tokensUsed !== undefined) {
    parts.push(`${formatTokens(tokensUsed)} tokens`);
  }
  const rate = resolveTokensPerSecond(tokensUsed, elapsedMs, tokensPerSecond);
  if (rate !== null) {
    parts.push(formatTokensPerSecond(rate));
  }
  return parts.join(" · ");
}
