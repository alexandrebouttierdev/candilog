import { useEffect, useId, useRef, useState } from "react";
import { useQuery } from "@tanstack/react-query";
import { Icon } from "./Icon";
import { controlClasses } from "./FormField";
import { Pager } from "./Pager";
import { cn } from "@/shared/lib/cn";
import { useDebounce } from "@/shared/hooks/useDebounce";
import type { Page } from "@/shared/types/page";

/** Ce que le sélecteur affiche d'une entité, indépendamment de son type. */
export interface EntityOption {
  readonly id: string;
  readonly label: string;
  readonly meta?: string | undefined;
}

/**
 * Sélecteur d'entité avec recherche et pagination.
 *
 * Un `select` natif ne convient pas ici : il exige de charger toute la collection, ce que le
 * guide interdit au-delà de cinquante éléments. Le composant délègue donc la recherche et la
 * pagination à l'appelant, qui les transmet au backend — il ne reçoit qu'une page à la fois.
 */
export function EntityPicker({
  value,
  selectedLabel,
  placeholder,
  emptyHelp,
  page_size = 4,
  invalid = false,
  describedBy,
  id,
  onChange,
  fetchPage,
  queryKey,
}: {
  /** Id sélectionné, ou `null`. */
  value: string | null;
  /** Libellé de la sélection courante, connu de l'appelant seul. */
  selectedLabel: string | null;
  placeholder: string;
  /** Aide affichée sous la liste quand la recherche ne donne rien. */
  emptyHelp?: string;
  page_size?: number;
  invalid?: boolean;
  describedBy?: string | undefined;
  id?: string;
  onChange: (id: string | null) => void;
  /** Payload une page de résultats pour une recherche donnée. */
  fetchPage: (params: { page: number; page_size: number; search: string }) => Promise<Page<EntityOption>>;
  /** Root de la clé de cache, propre à l'entité recherchée. */
  queryKey: readonly unknown[];
}) {
  const [ouvert, setOuvert] = useState(false);
  const [saisie, setSaisie] = useState("");
  const [page, setPage] = useState(1);
  const recherche = useDebounce(saisie);
  const conteneur = useRef<HTMLDivElement>(null);
  const listId = useId();

  const resultats = useQuery({
    queryKey: [...queryKey, "picker", { page, page_size, recherche }],
    queryFn: () => fetchPage({ page, page_size, search: recherche }),
    enabled: ouvert,
  });

  // Un clic ailleurs referme la liste. Sans cela, elle resterait ouverte par-dessus les
  // champs suivants du formulaire, qu'elle rendrait inatteignables à la souris.
  useEffect(() => {
    if (!ouvert) return;
    const surClic = (event: MouseEvent) => {
      if (!conteneur.current?.contains(event.target as Node)) setOuvert(false);
    };
    document.addEventListener("mousedown", surClic);
    return () => document.removeEventListener("mousedown", surClic);
  }, [ouvert]);

  const items = resultats.data?.items ?? [];

  return (
    <div ref={conteneur} className="relative">
      <div className="relative">
        <Icon
          name="search"
          size={16}
          className="pointer-events-none absolute top-1/2 left-2.5 -translate-y-1/2 text-ink-faint"
        />
        <input
          id={id}
          type="text"
          role="combobox"
          aria-expanded={ouvert}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-invalid={invalid}
          aria-describedby={describedBy}
          value={ouvert ? saisie : (selectedLabel ?? "")}
          placeholder={placeholder}
          onFocus={() => {
            setOuvert(true);
            setSaisie("");
          }}
          onChange={(event) => {
            setSaisie(event.target.value);
            // Toute nouvelle recherche ramène en première page : rester en page 2 après
            // avoir restreint la recherche afficherait une liste vide alors que des
            // résultats existent. Fait ici plutôt que dans un effet sur la valeur
            // retardée, qui provoquerait un rendu en cascade.
            setPage(1);
          }}
          className={controlClasses(invalid, value && !ouvert ? "pr-9 pl-8" : "pl-8")}
        />
        {value && !ouvert ? (
          <button
            type="button"
            aria-label="Effacer la sélection"
            onClick={() => onChange(null)}
            className="absolute top-1/2 right-2 -translate-y-1/2 rounded-button p-1 text-ink-faint transition-colors duration-150 hover:bg-neutral-tint hover:text-ink"
          >
            <Icon name="close" size={15} />
          </button>
        ) : null}
      </div>

      {ouvert ? (
        <div
          id={listId}
          role="listbox"
          className="absolute top-full right-0 left-0 z-20 mt-1 overflow-hidden rounded-field border border-line bg-surface shadow-e2"
        >
          {resultats.isPending ? (
            <p className="px-3 py-3 text-meta text-ink-faint">Recherche…</p>
          ) : items.length === 0 ? (
            <p className="px-3 py-3 text-meta text-ink-faint">
              {emptyHelp ?? "Aucun résultat."}
            </p>
          ) : (
            <ul className="max-h-56 overflow-y-auto">
              {items.map((option) => (
                <li key={option.id}>
                  <button
                    type="button"
                    role="option"
                    aria-selected={option.id === value}
                    onClick={() => {
                      onChange(option.id);
                      setOuvert(false);
                    }}
                    className={cn(
                      "flex w-full items-center gap-2 px-3 py-2 text-left transition-colors duration-150",
                      option.id === value ? "bg-accent-tint" : "hover:bg-neutral-tint",
                    )}
                  >
                    <span className="min-w-0 flex-1">
                      <span className="block truncate text-body text-ink">{option.label}</span>
                      {option.meta ? (
                        <span className="block truncate text-meta text-ink-muted">
                          {option.meta}
                        </span>
                      ) : null}
                    </span>
                    {option.id === value ? (
                      <Icon name="check" size={15} className="flex-none text-accent" />
                    ) : null}
                  </button>
                </li>
              ))}
            </ul>
          )}

          {(resultats.data?.total ?? 0) > page_size ? (
            <Pager
              page={page}
              page_size={page_size}
              total={resultats.data?.total ?? 0}
              label="résultats"
              onPageChange={setPage}
            />
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
