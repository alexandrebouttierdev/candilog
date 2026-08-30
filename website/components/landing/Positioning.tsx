import { BrandIcon } from "@/components/ui/BrandIcon";
import { Icon } from "@/components/ui/Icon";
import { Reveal } from "@/components/ui/Reveal";
import { cn } from "@/lib/cn";

/* Grille 2×2 des quatre faits (§7.3). Les filets internes ne peuvent pas être
   des bordures uniformes : la 1re cellule n'a ni filet gauche ni filet haut, la 2e
   n'a que le gauche, etc. Les paddings sont asymétriques pour la même raison —
   le texte affleure les bords extérieurs de la grille. */
const FAITS = [
  {
    icone: "work_history" as const,
    accentuee: true,
    titre: "Un suivi de recherche",
    detail: "Candidatures, entreprises, entretiens",
  },
  {
    icone: "computer" as const,
    accentuee: false,
    titre: "Une vraie application",
    detail: "Installée, rapide, utilisable hors ligne",
  },
  {
    icone: "folder_managed" as const,
    accentuee: false,
    titre: "Vos données chez vous",
    detail: "Sur votre machine, exportables",
  },
  {
    icone: "github" as const,
    accentuee: false,
    titre: "Code source disponible",
    detail: "Consultable publiquement sur GitHub",
  },
];

export function Positioning() {
  return (
    <section className="border-b border-line bg-page">
      <Reveal className="mx-auto grid max-w-[1240px] grid-cols-[repeat(auto-fit,minmax(min(340px,100%),1fr))] items-start gap-[clamp(28px,5vw,72px)] px-[clamp(16px,4vw,40px)] py-[clamp(48px,6vw,84px)]">
        <p className="max-w-[560px] text-pretty text-[clamp(17px,1.5vw,20px)] leading-[1.65] text-ink">
          Candilog regroupe ce qu&apos;une recherche d&apos;emploi demande au
          quotidien : les offres qui vous intéressent, les candidatures
          envoyées, les réponses à relancer, les entretiens à préparer et les
          documents que vous adaptez pour chaque poste.
        </p>

        {/* Deux colonnes fixes : en auto-fit les filets se désalignent (§6). */}
        <div className="grid min-w-0 max-w-[460px] grid-cols-[repeat(2,minmax(0,1fr))] border-t border-control">
          {FAITS.map((fait, index) => {
            const colonneDroite = index % 2 === 1;
            const rangeeBasse = index >= 2;
            return (
              <div
                key={fait.titre}
                className={cn(
                  "py-4",
                  colonneDroite
                    ? "border-l border-line pl-[18px] pr-0"
                    : "pl-0 pr-[18px]",
                  rangeeBasse && "border-t border-line",
                )}
              >
                {fait.icone === "github" ? (
                  <span className="block text-ink-muted">
                    <BrandIcon name="github" size={19} />
                  </span>
                ) : (
                  <span
                    className={cn(
                      "block",
                      fait.accentuee ? "text-accent" : "text-ink-muted",
                    )}
                  >
                    <Icon name={fait.icone} size={20} />
                  </span>
                )}
                <p className="mt-[9px] text-[13px] font-semibold text-ink">
                  {fait.titre}
                </p>
                <p className="mt-[3px] text-[12.5px] leading-[1.5] text-ink-tertiary">
                  {fait.detail}
                </p>
              </div>
            );
          })}
        </div>
      </Reveal>
    </section>
  );
}
