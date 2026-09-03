import { ipc } from "./ipc";
import { AppError } from "@/shared/types/app-error";
import { useUiStore } from "@/shared/lib/ui-store";

/**
 * Ouvre une URL dans le navigateur du système.
 *
 * Un `<a target="_blank">` ou un `window.open` ne fait rien dans la WebView Tauri : la
 * fenêtre n'a pas de gestionnaire de popup, et la capability `opener` reste restreinte à
 * deux origines. Le lien passe donc par une commande Rust qui valide le schéma
 * (`core::browser`) avant de déléguer au lanceur système.
 *
 * L'échec est annoncé à l'utilisateur : un lien qui ne s'ouvre pas sans un mot laisse
 * croire à un clic manqué. Fonction plutôt que hook — appelée depuis de simples
 * gestionnaires de clic — d'où `getState()` plutôt que le hook React.
 */
export async function openExternal(url: string): Promise<void> {
  try {
    await ipc<void>("open_external_url", { url });
  } catch (error) {
    useUiStore.getState().notify({
      tone: "error",
      title: "Lien impossible à ouvrir",
      detail: error instanceof AppError ? error.message : undefined,
    });
  }
}
