import { FauxBouton, FauxSwitch, StatusGlyph } from "@/components/landing/app/primitives";
import { Reveal } from "@/components/ui/Reveal";

/* Faits vérifiés dans le code : base locale, sauvegarde et restauration
   (`useBackupsViewModel`), coffre système pour les clés, sorties réseau limitées
   (`docs/CLAUDE_DESIGN_CONTEXT.md` §1). Formulation alignée sur la FAQ calibrée. */
const FAITS = [
  ["Aucun compte", "Pas d'inscription, pas de serveur Candilog, pas de synchronisation."],
  [
    "Sauvegarde en un fichier",
    "Sauvegarder, restaurer ou tout réinitialiser depuis les Réglages. Une restauration est vérifiée avant de remplacer quoi que ce soit.",
  ],
  ["Clés d'API protégées", "Rangées dans le trousseau de votre système, jamais réaffichées en clair."],
  [
    "Connexions sortantes limitées",
    "Le fournisseur d'IA que vous choisissez, la vérification des mises à jour, et les liens que vous ouvrez.",
  ],
] as const;

/** Dialogue du premier envoi distant (`RemoteSendDialog`), texte repris du composant. */
function DialogueEnvoi() {
  return (
    <div className="flex w-full max-w-[396px] flex-col gap-[14px] rounded-r11 border border-bd-menu bg-modal p-5 shadow-pop md:p-[22px]">
      <span className="flex items-start gap-[10px]">
        <StatusGlyph ton="a" className="mt-[5px]" />
        <span className="serif-title text-[16.5px] leading-[1.3]">Cette tâche sera envoyée à Anthropic</span>
      </span>
      <span className="text-[12.5px] leading-[1.55] text-tx-3">
        Ce qui quitte votre ordinateur pour « Analyser un CV » : votre CV et le texte de
        l&apos;offre. Anthropic le traite selon sa propre politique. Ce que les autres tâches
        confient à l&apos;IA locale reste sur votre machine.
      </span>
      <span className="flex flex-col border-t border-bd-soft text-[12px]">
        {[
          ["Destinataire", "Anthropic"],
          ["Tâche", "Analyser un CV"],
          ["Tâches concernées", "1 sur 5"],
        ].map(([libelle, valeur]) => (
          <span key={libelle} className="flex justify-between border-b border-bd-soft py-[7px]">
            <span className="text-tx-4">{libelle}</span>
            <span>{valeur}</span>
          </span>
        ))}
      </span>
      <span className="flex items-center gap-[10px] rounded-r8 bg-group px-3 py-2 text-[12px] text-tx-2">
        <FauxSwitch actif={false} />
        Ne plus demander pour Anthropic
      </span>
      <span className="flex items-center justify-between gap-3">
        <span className="font-mono text-[10px] text-tx-5">demandé une seule fois par service</span>
        <span className="flex gap-[6px]">
          <FauxBouton taille="dialogue" className="shadow-[0_0_0_1px_var(--ac)]">Annuler</FauxBouton>
          <FauxBouton variante="primaire" taille="dialogue">
            Envoyer
          </FauxBouton>
        </span>
      </span>
    </div>
  );
}

/** Section Confidentialité : ce qui est réellement implémenté, rien de plus. Le
 *  paragraphe reprend la réponse calibrée de la FAQ. */
export function Privacy() {
  return (
    <section id="confidentialite" aria-labelledby="confidentialite-titre">
      <Reveal className="mx-auto grid max-w-[1200px] items-center gap-12 px-4 py-16 md:px-8 md:py-24 lg:grid-cols-[minmax(0,1fr)_400px] lg:gap-16 xl:grid-cols-[minmax(0,1fr)_460px] xl:gap-24 xl:px-0 xl:py-32">
        <div className="flex flex-col gap-[22px]">
          <span className="eyebrow">Confidentialité</span>
          <h2
            id="confidentialite-titre"
            className="serif-title text-balance text-[32px] leading-[1.1] tracking-[-0.025em] md:text-[40px] xl:text-[44px] xl:leading-[1.08]"
          >
            Vos données sont
            <br className="hidden sm:block" /> sur votre ordinateur.
          </h2>
          <p className="max-w-[560px] text-pretty text-[16px] leading-[1.65] text-tx-3">
            Vos candidatures, documents et notes sont enregistrés sur votre ordinateur. Si vous
            connectez un fournisseur d&apos;IA en ligne, le contenu nécessaire à la tâche demandée
            lui est transmis au moment où vous la lancez. Avec l&apos;IA locale, aucune donnée
            n&apos;est envoyée à un service externe.
          </p>
          <dl className="m-0 flex flex-col border-t border-bd">
            {FAITS.map(([terme, definition]) => (
              <div key={terme} className="grid gap-1 border-b border-bd py-[14px] sm:grid-cols-[180px_minmax(0,1fr)] sm:gap-6 xl:grid-cols-[220px_minmax(0,1fr)]">
                <dt className="text-[14px] font-semibold">{terme}</dt>
                <dd className="m-0 text-[14px] leading-[1.55] text-tx-3">{definition}</dd>
              </div>
            ))}
          </dl>
        </div>
        <figure className="m-0 flex justify-center rounded-r12 bg-desk px-4 py-8 md:px-8 md:py-11">
          <figcaption className="sr-only">
            Dialogue affiché par Candilog avant le premier envoi à un service distant : il
            indique le destinataire, la tâche et ce qui quitte l&apos;ordinateur, avec
            l&apos;option « Ne plus demander » désactivée par défaut.
          </figcaption>
          <div aria-hidden="true" className="flex w-full justify-center">
            <DialogueEnvoi />
          </div>
        </figure>
      </Reveal>
    </section>
  );
}
