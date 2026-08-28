import { PageHeader, EmptyState } from "@/shared/ui";

/**
 * Jalon d'écran non encore migré.
 *
 * Volontairement explicite plutôt que joli : un écran qui se contenterait d'être vide
 * serait indiscernable d'un écran en panne, et des données factices contreviendraient au
 * §52 de `MIGRATION.md`.
 */
export function PlaceholderPage({
  icon,
  title,
  section,
}: {
  icon: string;
  title: string;
  section: string;
}) {
  return (
    <div className="flex h-full flex-col">
      <PageHeader icon={icon} title={title} subtitle={section} />
      <div className="flex flex-1 items-center justify-center">
        <EmptyState
          icon="construction"
          title="Écran non encore migré"
          description="Cet écran sera branché sur les données réelles du backend Rust lors de la tranche de migration correspondante."
        />
      </div>
    </div>
  );
}
