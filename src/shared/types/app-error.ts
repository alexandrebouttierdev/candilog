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
 * Error applicative telle que la voit le frontend.
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

  /**
   * Détail technique, hors du message affiché.
   *
   * Renseigné quand l'IPC échoue avant d'atteindre une commande : le texte brut de Tauri est
   * en anglais et nomme des composants internes (« Command … not found »), ce que ni §1
   * (messages utilisateur en français) ni §13 (les détails internes restent hors de l'UI) ne
   * tolèrent dans un toast. Il reste disponible ici pour le diagnostic, en position
   * secondaire, sans être le message principal.
   */
  readonly detail: string | undefined;

  constructor(dto: AppErrorDto, detail?: string) {
    super(dto.message);
    this.name = "AppError";
    this.code = dto.code;
    this.detail = detail;
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
  return new AppError(
    {
      code: "IPC_ERROR",
      message: "Une erreur inattendue est survenue lors de la communication avec Candilog.",
    },
    // La chaîne brute n'est plus le message affiché : elle est en anglais et nomme des
    // composants internes. Elle reste accessible en détail, pour le diagnostic.
    typeof value === "string" ? value : undefined,
  );
}
