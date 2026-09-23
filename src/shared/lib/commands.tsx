import type { ReactNode } from "react";
import { createContext, useContext, useEffect, useMemo, useRef, useState } from "react";

/**
 * Groupe d'une commande de la palette (`screens/20-command-palette.png`) : les actions sur
 * la sélection d'abord, puis créer, aller à, réglages.
 */
export type CommandGroup = "selection" | "create" | "goto" | "settings";

export interface Command {
  readonly id: string;
  readonly group: CommandGroup;
  readonly label: string;
  /** Précision à droite du libellé : objet visé, localité de l'IA, décompte. */
  readonly detail?: string | undefined;
  /**
   * Glyphe typographique du **genre** de la commande (`↻ ▲ ✦ ✎ ◫ ◷ ▤`, `DECISIONS.md` G4).
   * Décoratif : le libellé porte l'information.
   */
  readonly glyph?: string;
  /** Raccourci imprimé à droite (notation neutre, séquences séparées par une espace). */
  readonly shortcut?: string;
  /** Mots supplémentaires pour la recherche. */
  readonly keywords?: string;
  readonly run: () => void;
}

interface RegistryActions {
  readonly register: (owner: symbol, commands: readonly Command[]) => void;
  readonly unregister: (owner: symbol) => void;
}

const CommandList = createContext<readonly Command[]>([]);
const CommandActions = createContext<RegistryActions>({
  register: () => undefined,
  unregister: () => undefined,
});

/**
 * Fournit le registre ; monté par la coque autour des écrans et de la palette.
 *
 * Les actions vivent dans un contexte stable, séparé de la liste : un écran qui s'inscrit
 * ne doit pas voir son effet relancé chaque fois que la liste change, sans quoi inscription
 * et rendu se relanceraient l'un l'autre.
 */
export function CommandProvider({ children }: { children: ReactNode }) {
  const [owners, setOwners] = useState<ReadonlyMap<symbol, readonly Command[]>>(new Map());
  const actions = useMemo<RegistryActions>(
    () => ({
      register: (owner, commands) => setOwners((current) => new Map(current).set(owner, commands)),
      unregister: (owner) =>
        setOwners((current) => {
          const next = new Map(current);
          next.delete(owner);
          return next;
        }),
    }),
    [],
  );
  const commands = useMemo(() => [...owners.values()].flat(), [owners]);
  return (
    <CommandActions value={actions}>
      <CommandList value={commands}>{children}</CommandList>
    </CommandActions>
  );
}

export function useCommands(): readonly Command[] {
  return useContext(CommandList);
}

/** Ce qui se voit d'une commande : si rien de cela ne change, la liste inscrite reste. */
function signatureOf(commands: readonly Command[]): string {
  return commands
    .map((command) =>
      [command.id, command.group, command.label, command.detail ?? "", command.glyph ?? "", command.shortcut ?? "", command.keywords ?? ""].join("\u0001"),
    )
    .join("\u0002");
}

/**
 * Ajoute des commandes à la palette tant que le composant est monté.
 *
 * L'appelant peut reconstruire sa liste à chaque rendu : elle n'est réinscrite que si ce
 * qu'elle **affiche** change, et `run` exécute toujours la dernière version reçue — un
 * geste lancé depuis la palette voit donc l'état courant de l'écran, pas celui du moment
 * de l'inscription.
 */
export function useRegisterCommands(commands: readonly Command[]) {
  const { register, unregister } = useContext(CommandActions);
  const [owner] = useState(() => Symbol("commandes"));
  const latest = useRef(commands);
  useEffect(() => {
    latest.current = commands;
  });
  const signature = signatureOf(commands);
  useEffect(() => {
    register(
      owner,
      latest.current.map((command) => ({
        ...command,
        run: () => latest.current.find((item) => item.id === command.id)?.run(),
      })),
    );
  }, [owner, signature, register]);
  useEffect(() => () => unregister(owner), [owner, unregister]);
}

/** Forme de recherche : sans accents, sans casse, espaces compactés. */
export function searchKey(value: string): string {
  return value
    .normalize("NFD")
    .replace(/\p{Diacritic}/gu, "")
    .toLowerCase()
    .replace(/\s+/g, " ")
    .trim();
}

/** Commandes qui contiennent chaque mot de la requête, dans le libellé, la précision ou les mots-clés. */
export function filterCommands(commands: readonly Command[], query: string): readonly Command[] {
  const words = searchKey(query).split(" ").filter(Boolean);
  if (words.length === 0) return commands;
  return commands.filter((command) => {
    const haystack = searchKey(`${command.label} ${command.detail ?? ""} ${command.keywords ?? ""}`);
    return words.every((word) => haystack.includes(word));
  });
}
