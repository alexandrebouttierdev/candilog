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
  onCreate,
  createLabel,
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
  /**
   * Ouvre la création de l'entité recherchée, avec le texte saisi.
   *
   * Sans cela, ne pas trouver une entité oblige à quitter le formulaire en cours pour aller
   * la créer ailleurs, puis à tout ressaisir.
   */
  onCreate?: (search: string) => void;
  /** Verbe de l'action de création, complété par le texte recherché. */
  createLabel?: string;
  /** Payload une page de résultats pour une recherche donnée. */
  fetchPage: (params: { page: number; page_size: number; search: string }) => Promise<Page<EntityOption>>;
  /** Root de la clé de cache, propre à l'entité recherchée. */
  queryKey: readonly unknown[];
}) {
  const [isOpen, setOpen] = useState(false);
  // `null` tant que l'utilisateur n'a rien tapé : le champ affiche alors la sélection
  // courante. Sans cette distinction, ouvrir un formulaire prérempli effaçait à l'écran la
  // valeur déjà choisie, puisque le simple focus vidait la saisie.
  const [draft, setDraft] = useState<string | null>(null);
  const [page, setPage] = useState(1);
  const [selectedOption, setSelectedOption] = useState<EntityOption | null>(null);
  const search = useDebounce(draft ?? "");
  const container = useRef<HTMLDivElement>(null);
  const listId = useId();

  const results = useQuery({
    queryKey: [...queryKey, "picker", { page, page_size, search }],
    queryFn: () => fetchPage({ page, page_size, search: search }),
    enabled: isOpen,
  });

  // Un clic ailleurs referme la liste. Sans cela, elle resterait ouverte par-dessus les
  // champs suivants du formulaire, qu'elle rendrait inatteignables à la souris.
  useEffect(() => {
    if (!isOpen) return;
    const onOutsideClick = (event: MouseEvent) => {
      if (container.current?.contains(event.target as Node)) return;
      setOpen(false);
      setDraft(null);
    };
    document.addEventListener("mousedown", onOutsideClick);
    return () => document.removeEventListener("mousedown", onOutsideClick);
  }, [isOpen]);

  const items = results.data?.items ?? [];
  /** Libellé de la sélection, éventuellement connu du seul appelant. */
  const selectionLabel =
    (selectedOption?.id === value ? selectedOption.label : selectedLabel) ?? "";
  // Le texte proposé à la création est la draft immédiate, pas la valeur retardée : le
  // bouton doit nommer ce que l'utilisateur voit dans le champ.
  const toCreate = (draft ?? "").trim();
  const canCreate = onCreate !== undefined && toCreate.length > 0;

  return (
    <div ref={container} className="relative">
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
          aria-expanded={isOpen}
          aria-controls={listId}
          aria-autocomplete="list"
          aria-invalid={invalid}
          aria-describedby={describedBy}
          value={draft ?? selectionLabel}
          placeholder={placeholder}
          onFocus={(event) => {
            setOpen(true);
            // La sélection reste lisible ; le texte est présélectionné pour que la première
            // frappe la remplace entièrement, comme dans un champ de search natif.
            event.currentTarget.select();
          }}
          onChange={(event) => {
            setDraft(event.target.value);
            // Toute nouvelle search ramène en première page : rester en page 2 après
            // avoir restreint la search afficherait une liste vide alors que des
            // résultats existent. Fait ici plutôt que dans un effet sur la valeur
            // retardée, qui provoquerait un rendu en cascade.
            setPage(1);
          }}
          className={controlClasses(invalid, value && !isOpen ? "pr-9 pl-8" : "pl-8")}
        />
        {value && !isOpen ? (
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

      {isOpen ? (
        <div
          id={listId}
          role="listbox"
          className="absolute top-full right-0 left-0 z-20 mt-1 overflow-hidden rounded-field border border-line bg-surface shadow-e2"
        >
          {results.isPending ? (
            <p className="px-3 py-3 text-meta text-ink-faint">Recherche…</p>
          ) : results.isError ? (
            // Un échec de la commande n'est pas une absence de résultat : annoncer « Aucun
            // résultat » inviterait à créer un doublon d'une entité déjà enregistrée.
            <p className="px-3 py-3 text-meta text-danger">La search a échoué.</p>
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
                      setSelectedOption(option);
                      onChange(option.id);
                      setOpen(false);
                      setDraft(null);
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

          {canCreate && !results.isError ? (
            <button
              type="button"
              onClick={() => {
                setOpen(false);
                setDraft(null);
                onCreate(toCreate);
              }}
              className="flex w-full items-center gap-2 border-t border-line px-3 py-2 text-left text-body text-accent-text transition-colors duration-150 hover:bg-accent-tint"
            >
              <Icon name="add" size={15} className="flex-none" />
              <span className="min-w-0 flex-1 truncate">{`${createLabel ?? "Créer"} « ${toCreate} »`}</span>
            </button>
          ) : null}

          {(results.data?.total ?? 0) > page_size ? (
            <Pager
              page={page}
              page_size={page_size}
              total={results.data?.total ?? 0}
              label="résultats"
              onPageChange={setPage}
            />
          ) : null}
        </div>
      ) : null}
    </div>
  );
}
