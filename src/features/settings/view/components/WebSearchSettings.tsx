import { useState } from "react";
import { Button, TextInput } from "@/shared/ui";
import { useWebSearchSettings } from "../../viewmodel/useWebSearchSettings";
import { SettingsRow } from "./SettingsSection";

/**
 * Ligne « Recherche Web (SearXNG) » des Réglages.
 *
 * Les instances publiques acceptent rarement toutes les requêtes automatiques : désigner
 * la sienne (ou une instance de confiance) rend « Compléter avec l'IA » plus fiable.
 */
export function WebSearchSettings() {
  const vm = useWebSearchSettings();
  const [draft, setDraft] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const value = draft ?? vm.current;
  return (
    <SettingsRow
      label="Recherche Web (SearXNG)"
      hint="Utilisée par « Compléter avec l'IA ». Laissez vide pour essayer les instances publiques de searx.space ; indiquez la vôtre si elles refusent les recherches."
    >
      <form
        className="flex items-start gap-2"
        onSubmit={(event) => {
          event.preventDefault();
          const message = vm.submit(value);
          setError(message);
          if (message === null) setDraft(null);
        }}
      >
        <div className="w-[260px]">
          <TextInput
            aria-label="Adresse de l'instance SearXNG"
            placeholder="https://searx.exemple.fr"
            value={value}
            invalid={error !== null}
            disabled={!vm.ready}
            onChange={(event) => setDraft(event.target.value)}
          />
          {error ? <p role="alert" className="mt-1 text-tiny text-st-c">{error}</p> : null}
        </div>
        <Button type="submit" size="compact" disabled={!vm.ready || vm.isSaving || draft === null}>
          Enregistrer
        </Button>
      </form>
    </SettingsRow>
  );
}
