import { useNavigate } from "react-router-dom";
import { PATHS } from "@/shared/lib/paths";
import { Banner, Button } from "@/shared/ui";
import { enumerate, type ProfileGaps } from "../../model/profileReadiness";

/**
 * Avertissement d'un profil trop vide pour nourrir une génération (`docs/DESIGN.md` §12).
 *
 * En tête du panneau de gauche, là où le brief se règle : l'utilisateur voit ce qui manque
 * avant de lancer, pas après avoir lu une feuille creuse. Ambre et non rouge — rien n'a
 * échoué, et générer reste possible.
 *
 * « Compléter le profil » quitte la surcouche : rien n'est encore généré à cet instant, et
 * revenir sans son profil n'aurait servi à rien.
 */
export function ProfileGapBanner({ gaps, what }: { gaps: ProfileGaps; what: "CV" | "lettre" }) {
  const navigate = useNavigate();
  if (gaps.length === 0) return null;
  return (
    <Banner
      tone="warning"
      title="Profil à compléter"
      className="mb-4"
      message={
        <>
          Candilog écrit {what === "CV" ? "un CV" : "une lettre"} à partir de votre profil : il y manque{" "}
          {enumerate(gaps)}.
          {/* `flex` plutôt qu'une marge seule : le bouton est en ligne dans le paragraphe
              du bandeau, et il lui faut sa propre ligne dans une colonne de 234 px. */}
          <span className="mt-2.5 flex">
            <Button variant="secondary" size="compact" onClick={() => void navigate(PATHS.profile)}>
              Compléter le profil
            </Button>
          </span>
        </>
      }
    />
  );
}
