import { invoke } from "@tauri-apps/api/core";
import { toAppError } from "@/shared/types/app-error";

/**
 * Unique point d'appel de l'IPC Tauri du frontend.
 *
 * Les services de feature passent par cette fonction ; ni les vues, ni les ViewModels
 * n'appellent `invoke` directement (docs/CODE_RULES.md §4). Deux bénéfices concrets : toute
 * erreur remontée à React est déjà normalisée en `AppError` avec son code, et la liste
 * des commandes réellement utilisées reste lisible en cherchant les appelants de `ipc`.
 */
export async function ipc<TResult>(
  command: string,
  args?: Record<string, unknown>,
): Promise<TResult> {
  try {
    return await invoke<TResult>(command, args);
  } catch (error) {
    throw toAppError(error);
  }
}
