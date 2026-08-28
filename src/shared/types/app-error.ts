import type { AppErrorDto } from "./generated/app-error";

export type { AppErrorDto };

/**
 * Codes d'erreur émis par le backend Rust (`AppError::code`).
 *
 * Le frontend branche son comportement sur le code et jamais sur le texte du message :
 * celui-ci est rédigé pour l'utilisateur et peut être reformulé sans préavis.
 */
export const APP_ERROR_CODES = [
  "VALIDATION_ERROR",
  "NOT_FOUND",
  "DATABASE_ERROR",
  "HTTP_ERROR",
  "SERIALIZATION_ERROR",
  "PROVIDER_ERROR",
  "CANCELLED",
  /** Émis par le frontend lorsque l'IPC échoue avant d'atteindre une commande. */
  "IPC_ERROR",
] as const;

export type AppErrorCode = (typeof APP_ERROR_CODES)[number];

/**
 * Erreur applicative telle que la voit le frontend.
 *
 * Étend `Error` pour rester interceptable par les frontières usuelles (TanStack Query,
 * error boundaries) tout en conservant le code structuré.
 */
export class AppError extends Error {
  /**
   * Code remonté par le backend.
   *
   * Typé `string` et non `AppErrorCode` : la valeur traverse une frontière IPC et peut
   * venir d'un backend plus récent que le frontend. La restreindre à l'union ferait mentir
   * le type au premier code ajouté côté Rust ; `AppErrorCode` sert à écrire les
   * comparaisons, pas à décrire ce qui arrive réellement.
   */
  readonly code: string;

  constructor(dto: AppErrorDto) {
    super(dto.message);
    this.name = "AppError";
    this.code = dto.code;
  }

  /** L'utilisateur a annulé : à ignorer silencieusement plutôt qu'à signaler. */
  get isCancelled(): boolean {
    return this.code === "CANCELLED";
  }
}

/** Une valeur rejetée par l'IPC est-elle bien un `AppErrorDto` ? */
function isAppErrorDto(value: unknown): value is AppErrorDto {
  return (
    typeof value === "object" &&
    value !== null &&
    typeof (value as AppErrorDto).code === "string" &&
    typeof (value as AppErrorDto).message === "string"
  );
}

/**
 * Normalise ce que rejette `invoke` en `AppError`.
 *
 * Le backend rejette toujours un `AppErrorDto`, mais l'IPC lui-même peut échouer en amont
 * (commande inconnue, permission refusée par les capabilities, arguments non
 * désérialisables) : ces cas remontent une chaîne brute qu'il faut malgré tout présenter.
 */
export function toAppError(value: unknown): AppError {
  if (value instanceof AppError) return value;
  if (isAppErrorDto(value)) return new AppError(value);
  return new AppError({
    code: "IPC_ERROR",
    message:
      typeof value === "string"
        ? value
        : "Une erreur inattendue est survenue lors de la communication avec Candilog.",
  });
}
