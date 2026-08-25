import { Icon } from "./Icon";
import { Button } from "./Button";

/**
 * Bandeau d'erreur non bloquant.
 *
 * Le guide demande une erreur *dans* l'écran plutôt qu'à la place de l'écran : ce qui a pu
 * être chargé reste visible, et l'utilisateur garde une action — « Réessayer » — au lieu
 * d'une page morte.
 */
export function ErrorBanner({
  title = "Chargement impossible",
  message,
  onRetry,
}: {
  title?: string;
  message: string;
  onRetry?: () => void;
}) {
  return (
    <div
      role="alert"
      className="flex items-start gap-2.5 rounded-card border border-danger/30 bg-danger-tint px-4 py-3"
    >
      <Icon name="error" size={18} className="mt-px flex-none text-danger" />
      <div className="min-w-0 flex-1">
        <p className="text-section text-ink">{title}</p>
        <p className="mt-0.5 text-meta text-ink-muted">{message}</p>
      </div>
      {onRetry ? (
        <Button variant="secondary" icon="refresh" onClick={onRetry}>
          Réessayer
        </Button>
      ) : null}
    </div>
  );
}
