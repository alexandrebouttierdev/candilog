/**
 * Tuile de marque Candilog, identique à celle de l'application (`BrandMark`) : indigo
 * `#5B62F0` fixe dans les deux thèmes — c'est la marque, elle ne suit pas l'accent —
 * et « C » vectorisé en blanc.
 */
export function BrandMark({ size = 22, className }: { size?: number; className?: string }) {
  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 64 64"
      fill="none"
      className={className ?? "block flex-none"}
    >
      <rect width="64" height="64" rx="17" className="fill-brand" />
      <path
        fill="#ffffff"
        d="M41.6 41.9c-2.4 2.6-5.6 3.9-9.5 3.9-4.3 0-7.8-1.4-10.3-4.1-2.5-2.7-3.8-6.4-3.8-10.9 0-4.4 1.3-8 3.9-10.7 2.6-2.7 6-4.1 10.3-4.1 3.8 0 6.9 1.2 9.3 3.7l-3.6 4.2c-1.6-1.7-3.5-2.5-5.7-2.5-2.5 0-4.4.9-5.9 2.6-1.4 1.7-2.2 4-2.2 6.8 0 2.9.7 5.2 2.2 6.9 1.5 1.7 3.5 2.6 6 2.6 2.3 0 4.3-.9 5.9-2.7l3.4 4.4Z"
      />
    </svg>
  );
}
