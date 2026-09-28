import type { ReactNode } from "react";

/**
 * Section de la surcouche Réglages v2 (`screens/18-settings.png`) : titre serif 21 px et
 * phrase qui annonce l'**effet** des réglages, pas leur mécanisme.
 */
export function SettingsSection({
  title,
  description,
  children,
}: {
  title: string;
  description: string;
  children: ReactNode;
}) {
  return (
    <section aria-labelledby="reglages-titre">
      <h2 id="reglages-titre" className="serif-title text-screen text-tx">
        {title}
      </h2>
      <p className="mt-1.5 text-ui text-tx-4">{description}</p>
      <div className="mt-3">{children}</div>
    </section>
  );
}

/**
 * Ligne de réglage : libellé et conséquence à gauche, contrôle à droite, filet bas.
 * Chaque réglage annonce ce qu'il change (« Nombre de lignes visibles sans faire défiler »).
 */
export function SettingsRow({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <div className="flex min-h-[60px] items-center gap-4 border-b border-bd-soft py-3">
      <div className="min-w-0 flex-1">
        <p className="text-row text-tx">{label}</p>
        {hint ? <p className="mt-0.5 text-sub text-tx-5">{hint}</p> : null}
      </div>
      <div className="flex-none">{children}</div>
    </div>
  );
}
