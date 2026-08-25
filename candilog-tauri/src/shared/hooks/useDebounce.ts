import { useEffect, useState } from "react";

/**
 * Retarde la propagation d'une valeur qui change à chaque frappe.
 *
 * Sans cela, chaque caractère saisi dans un champ de recherche déclencherait un appel IPC
 * et une requête `SQLite` : sur une saisie de dix lettres, neuf résultats seraient calculés
 * pour être aussitôt remplacés.
 */
export function useDebounce<T>(valeur: T, delaiMs = 250): T {
  const [retardee, setRetardee] = useState(valeur);

  useEffect(() => {
    const minuteur = setTimeout(() => setRetardee(valeur), delaiMs);
    return () => clearTimeout(minuteur);
  }, [valeur, delaiMs]);

  return retardee;
}
