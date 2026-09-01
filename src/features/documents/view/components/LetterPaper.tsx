import { useEffect, useRef, useState, type ReactNode } from "react";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { profileService } from "@/features/profile/services/profileService";
import { PROFILE_KEY } from "@/features/profile/viewmodel/useProfileViewModel";
import type { Identity } from "@/shared/types/generated/profile";
import {
  letterDateLine,
  letterHeadline,
  letterJobTitleFromHeadline,
  letterSignature,
} from "../../model/letterLayout";
import { ResumeEditableText } from "./ResumeEditableText";

const DENSITY_STEPS: { fs: number; sp: number }[] = [
  { fs: 1, sp: 1 },
  { fs: 0.98, sp: 0.86 },
  { fs: 0.95, sp: 0.74 },
  { fs: 0.92, sp: 0.64 },
];

export type LetterPaperField =
  | "company"
  | "job_title"
  | "recipient"
  | "recipient_address"
  | "job_reference";

/** Champs d'identité obligatoires : les vider donne une chaîne vide, jamais `null`. */
type IdentityRequired = "first_name" | "name" | "email";
/** Champs d'identité facultatifs : vides, ils disparaissent de la feuille. */
type IdentityOptional = "title" | "address" | "city" | "phone";

export type LetterPaperFields = {
  company: string | null;
  job_title: string | null;
  recipient: string | null;
  recipient_address: string | null;
  job_reference: string | null;
};

/**
 * Feuille A4 de la lettre, portée du template fourni.
 *
 * L'identité vient du profil courant (lecture seule). Entreprise, poste, destinataire et
 * référence sont éditables sur le papier en mode édition ; les blocs vides sont omis en
 * lecture. Aperçu et PDF partagent la même hiérarchie.
 */
export function LetterPaper({
  fields,
  editable = false,
  children,
  onChange,
  onOverflowChange,
}: {
  fields: LetterPaperFields;
  editable?: boolean;
  children: ReactNode;
  onChange?: (field: LetterPaperField, value: string) => void;
  onOverflowChange?: (overflow: boolean) => void;
}) {
  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const queryClient = useQueryClient();
  const enregistre = useMutation({
    mutationFn: (identity: Identity) => {
      const courant = profile.data?.profile;
      if (!courant) throw new Error("Profil indisponible");
      return profileService.save({ ...courant, identity });
    },
    onSuccess: () => queryClient.invalidateQueries({ queryKey: PROFILE_KEY }),
  });
  // Brouillon local : la zone éditable prévient à chaque frappe, mais le profil n'est
  // écrit qu'à la sortie du champ. Le brouillon suit le profil tant qu'on n'y touche pas.
  const [brouillon, setBrouillon] = useState<Identity | null>(null);
  // La sortie du champ suit immédiatement la frappe : l'état n'est pas encore réappliqué
  // quand elle survient, d'où cette référence qui, elle, porte toujours la dernière saisie.
  const brouillonRef = useRef<Identity | null>(null);
  const identity = brouillon ?? profile.data?.profile.identity;
  const firstName = identity?.first_name?.trim() ?? "";
  const lastName = identity?.name?.trim() ?? "";
  const nom = letterSignature(firstName, lastName);
  const title = identity?.title?.trim() || null;
  const address = identity?.address?.trim() || null;
  const city = identity?.city?.trim() || null;
  const phone = identity?.phone?.trim() || null;
  const email = identity?.email?.trim() || null;

  const poser = (suivant: Identity) => {
    brouillonRef.current = suivant;
    setBrouillon(suivant);
  };
  const identiteEditable = editable && identity !== undefined;

  const modifier = (champ: IdentityRequired, valeur: string) => {
    if (identity) poser({ ...identity, [champ]: valeur });
  };
  const modifierFacultatif = (champ: IdentityOptional, valeur: string) => {
    if (identity) poser({ ...identity, [champ]: valeur.trim() === "" ? null : valeur });
  };
  const valider = () => {
    const saisie = brouillonRef.current;
    if (saisie) enregistre.mutate(saisie);
  };
  const paperRef = useRef<HTMLElement | null>(null);
  const contentRef = useRef<HTMLDivElement | null>(null);
  const [overflow, setOverflow] = useState(false);

  useEffect(() => {
    const page = paperRef.current;
    const content = contentRef.current;
    if (!page || !content) return;
    let arrete = false;
    const mesurer = () => {
      if (arrete) return;
      const depasse = ajusterDensite(page, content);
      setOverflow(depasse);
      onOverflowChange?.(depasse);
    };
    const polices = typeof globalThis.document !== "undefined" ? globalThis.document.fonts : undefined;
    const pret = polices ? polices.ready : Promise.resolve();
    void pret.then(mesurer);
    let observateur: ResizeObserver | undefined;
    if (typeof ResizeObserver !== "undefined") {
      observateur = new ResizeObserver(mesurer);
      observateur.observe(content);
    }
    return () => {
      arrete = true;
      observateur?.disconnect();
    };
  }, [fields, editable, children, identity, onOverflowChange]);

  const changer = (field: LetterPaperField) => (value: string) => onChange?.(field, value);
  const afficher = (value: string | null) => Boolean(value?.trim()) || editable;
  const headline = letterHeadline(fields.job_title);

  return (
    <>
      {overflow ? (
        <p className="letter-overflow-warning" data-print-hide>
          Lettre trop longue pour une page A4 : la typographie est déjà au seuil minimal. Retire un
          paragraphe ou resserre l'accroche.
        </p>
      ) : null}
      <article ref={paperRef} aria-label="Lettre de motivation" className="letter-paper">
        <aside className="letter-identity">
          <div className="flex flex-col gap-[calc(6px*var(--letter-sp))]">
            {firstName && lastName ? (
              <h1 className="letter-name">
                <ResumeEditableText
                  tag="span"
                  label="Prénom"
                  value={firstName}
                  editable={identiteEditable}
                  onChange={(value) => modifier("first_name", value)}
                  onCommit={valider}
                />
                <br />
                <ResumeEditableText
                  tag="span"
                  label="Nom"
                  value={lastName}
                  editable={identiteEditable}
                  onChange={(value) => modifier("name", value)}
                  onCommit={valider}
                />
              </h1>
            ) : (
              <h1 className="letter-name">{nom}</h1>
            )}
            {identiteEditable || title ? (
              <ResumeEditableText
                tag="p"
                className="letter-role"
                label="Titre du profil"
                value={title ?? ""}
                editable={identiteEditable}
                onChange={(value) => modifierFacultatif("title", value)}
                onCommit={valider}
              />
            ) : null}
          </div>
          <div className="mt-[calc(22px*var(--letter-sp))] flex flex-col gap-[calc(9px*var(--letter-sp))]">
            <Coordonnee
              label="Adresse"
              editable={identiteEditable}
              onCommit={valider}
              lignes={[
                { valeur: address, libelle: "Adresse postale", onChange: (v) => modifierFacultatif("address", v) },
                { valeur: city, libelle: "Ville", onChange: (v) => modifierFacultatif("city", v) },
              ]}
            />
            <Coordonnee
              label="Téléphone"
              editable={identiteEditable}
              onCommit={valider}
              lignes={[{ valeur: phone, libelle: "Téléphone", onChange: (v) => modifierFacultatif("phone", v) }]}
            />
            <Coordonnee
              label="Courriel"
              editable={identiteEditable}
              onCommit={valider}
              lignes={[{ valeur: email, libelle: "Courriel", onChange: (v) => modifier("email", v) }]}
            />
          </div>
          <p className="letter-attachment">
            Pièce jointe :<br />
            curriculum vitæ
          </p>
        </aside>
        <div ref={contentRef} className="letter-content">
          <div className="flex items-start justify-between gap-6">
            <div className="flex min-w-0 flex-1 flex-col gap-[calc(2px*var(--letter-sp))]">
              {afficher(fields.company) ? (
                <ResumeEditableText
                  tag="p"
                  label="Entreprise destinataire"
                  value={fields.company ?? ""}
                  editable={editable}
                  className="letter-company"
                  onChange={changer("company")}
                />
              ) : null}
              {afficher(fields.recipient) ? (
                <ResumeEditableText
                  tag="p"
                  label="Interlocuteur"
                  value={fields.recipient ?? ""}
                  editable={editable}
                  className="letter-recipient"
                  onChange={changer("recipient")}
                />
              ) : null}
              {afficher(fields.recipient_address) ? (
                <ResumeEditableText
                  tag="p"
                  label="Adresse du destinataire"
                  value={fields.recipient_address ?? ""}
                  editable={editable}
                  className="letter-recipient"
                  onChange={changer("recipient_address")}
                />
              ) : null}
            </div>
            <p className="letter-date">{letterDateLine(city)}</p>
          </div>
          <div className="flex flex-col gap-[calc(4px*var(--letter-sp))]">
            {afficher(fields.job_title) ? (
              <ResumeEditableText
                tag="p"
                label="Poste ciblé"
                value={headline ?? fields.job_title ?? ""}
                editable={editable}
                className="letter-headline"
                onChange={(value) => changer("job_title")(letterJobTitleFromHeadline(value))}
              />
            ) : null}
            {afficher(fields.job_reference) ? (
              <ResumeEditableText
                tag="p"
                label="Référence de l'offre"
                value={
                  fields.job_reference?.trim()
                    ? `Référence de l'offre : ${fields.job_reference}`
                    : ""
                }
                editable={editable}
                className="letter-reference"
                onChange={(value) => {
                  const prefixe = "Référence de l'offre : ";
                  changer("job_reference")(
                    value.startsWith(prefixe) ? value.slice(prefixe.length) : value,
                  );
                }}
              />
            ) : null}
          </div>
          {children}
          <p className="letter-signature">{nom}</p>
        </div>
      </article>
    </>
  );
}

/** Une ligne de coordonnée : sa valeur courante et ce qu'elle modifie dans le profil. */
type LigneCoordonnee = {
  valeur: string | null;
  libelle: string;
  onChange: (value: string) => void;
};

/**
 * Bloc de coordonnée de la colonne d'identité.
 *
 * En lecture, les lignes vides disparaissent, comme dans le template. En édition elles
 * restent visibles : sans cela, une coordonnée absente serait impossible à renseigner.
 */
function Coordonnee({
  label,
  lignes,
  editable,
  onCommit,
}: {
  label: string;
  lignes: LigneCoordonnee[];
  editable: boolean;
  onCommit: () => void;
}) {
  const visibles = editable ? lignes : lignes.filter((ligne) => (ligne.valeur ?? "").trim() !== "");
  if (visibles.length === 0) return null;
  return (
    <div className="flex flex-col gap-px">
      <span className="letter-coord-label">{label}</span>
      {visibles.map((ligne) => (
        <ResumeEditableText
          key={`${label}-${ligne.libelle}`}
          tag="span"
          className="letter-coord-value"
          label={ligne.libelle}
          value={ligne.valeur ?? ""}
          editable={editable}
          onChange={ligne.onChange}
          onCommit={onCommit}
        />
      ))}
    </div>
  );
}

function ajusterDensite(page: HTMLElement, content: HTMLElement): boolean {
  const appliquer = (position: number) => {
    page.style.setProperty("--letter-fs", String(DENSITY_STEPS[position]?.fs ?? 1));
    page.style.setProperty("--letter-sp", String(DENSITY_STEPS[position]?.sp ?? 1));
  };
  let index = 0;
  appliquer(index);
  const overflows = () => content.scrollHeight - content.clientHeight > 1;
  while (overflows() && index < DENSITY_STEPS.length - 1) appliquer(++index);
  return overflows();
}
