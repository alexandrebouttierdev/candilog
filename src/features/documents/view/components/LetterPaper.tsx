import type { ReactNode } from "react";
import { useQuery } from "@tanstack/react-query";
import { profileService } from "@/features/profile/services/profileService";
import { PROFILE_KEY } from "@/features/profile/viewmodel/useProfileViewModel";
import { letterDateLine, letterSignature, letterSubject } from "../../model/letterLayout";

/**
 * Feuille de la lettre, composée comme l'export PDF.
 *
 * L'aperçu portait jusqu'ici un en-tête inventé pour l'écran — un intitulé de poste en gros
 * et le nom de l'entreprise — que la page imprimée n'a jamais eu. Il reprend maintenant les
 * mêmes blocs, dans le même ordre et aux mêmes proportions : identité, lieu et date, objet,
 * corps, signature. L'identité vient du profil, comme à l'export.
 */
export function LetterPaper({
  jobTitle,
  company,
  children,
}: {
  jobTitle: string | null;
  company: string | null;
  children: ReactNode;
}) {
  const profile = useQuery({ queryKey: PROFILE_KEY, queryFn: profileService.load });
  const identity = profile.data?.profile.identity;
  const nom = letterSignature(identity?.first_name ?? "", identity?.name ?? "");
  const city = identity?.city ?? null;
  const email = identity?.email ?? "";

  return (
    <article aria-label="Lettre de motivation" className="letter-paper">
      <div className="letter-page">
        <p className="letter-name">{nom}</p>
        {city?.trim() ? <p className="letter-meta">{city}</p> : null}
        {email.trim() ? <p className="letter-meta">{email}</p> : null}
        <p className="letter-date">{letterDateLine(city)}</p>
        <p className="letter-subject">{letterSubject(jobTitle, company)}</p>
        {children}
        <p className="letter-signature">{nom}</p>
      </div>
    </article>
  );
}
