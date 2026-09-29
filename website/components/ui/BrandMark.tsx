/**
 * Marque Candilog : monogramme C + coche, identique à l'application (`logo-candilog.svg`).
 * Pas de tuile indigo : le fond vient du conteneur.
 */
export function BrandMark({ size = 22, className }: { size?: number; className?: string }) {
  const gradientId = "candilog-brand-mark";

  return (
    <svg
      aria-hidden="true"
      width={size}
      height={size}
      viewBox="0 0 96 96"
      fill="none"
      className={className ?? "block flex-none"}
    >
      <defs>
        <linearGradient
          id={gradientId}
          x1="18"
          y1="16"
          x2="80"
          y2="82"
          gradientUnits="userSpaceOnUse"
        >
          <stop offset="0" stopColor="#6b7cff" />
          <stop offset="0.52" stopColor="#4f5fe8" />
          <stop offset="1" stopColor="#3d4fd4" />
        </linearGradient>
      </defs>
      <g transform="translate(4.2 0)">
        <path
          d="M 67.5 24.5 A 33.5 33.5 0 1 0 67.5 71.5"
          stroke={`url(#${gradientId})`}
          strokeWidth="14"
          strokeLinecap="round"
        />
        <path
          d="M 59.5 55.5 L 68 64 L 80.5 48.5"
          stroke="#4FC27A"
          strokeWidth="8"
          strokeLinecap="round"
          strokeLinejoin="round"
        />
      </g>
    </svg>
  );
}
