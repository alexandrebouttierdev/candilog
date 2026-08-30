import type { Metadata } from "next";

import { LegalLayout } from "@/components/layout/LegalLayout";
import {
  CarteInfo,
  LegalCorps,
  LegalHero,
  LegalSection,
  LigneDefinition,
  ListeDefinitions,
  P,
  ValeurMono,
} from "@/components/legal/primitives";
import { BrandIcon } from "@/components/ui/BrandIcon";
import { ButtonLink } from "@/components/ui/Button";
import { Icon } from "@/components/ui/Icon";
import { CONTACT_EMAIL, GITHUB_REPO, LICENCE_POLYFORM } from "@/lib/data/liens";

export const metadata: Metadata = {
  title: "Licence — Candilog",
  description:
    "Candilog est un projet source available : usages non commerciaux sous PolyForm Noncommercial 1.0.0, usage commercial sur licence séparée.",
};

const BOUTON_SECONDAIRE = "mt-5 h-[34px] px-[14px] text-[13px]";

export default function Page() {
  return (
    <LegalLayout courante="licence">
      <LegalHero
        surTitre="Licence"
        titre="Un code source accessible, une utilisation encadrée."
        chapo="Le code source de Candilog est disponible publiquement. Son utilisation est encadrée par un modèle distinguant les usages autorisés non commerciaux des usages commerciaux."
      />

      <ListeDefinitions encadree>
        <LigneDefinition libelle="Modèle" encadree>
          Source available
        </LigneDefinition>
        <LigneDefinition libelle="Usage non commercial" encadree>
          PolyForm Noncommercial License 1.0.0
        </LigneDefinition>
        <LigneDefinition libelle="Identifiant SPDX" encadree>
          <ValeurMono>PolyForm-Noncommercial-1.0.0</ValeurMono>
        </LigneDefinition>
        <LigneDefinition libelle="Usage commercial" encadree derniere>
          Licence commerciale séparée
        </LigneDefinition>
      </ListeDefinitions>

      <LegalCorps>
        <LegalSection titre="Code source disponible" premiere>
          <P>Candilog est un projet source available.</P>
          <P>
            Vous pouvez consulter son code source, suivre son développement et contribuer au projet
            dans le respect des conditions applicables.
          </P>
          <ButtonLink
            href={GITHUB_REPO}
            target="_blank"
            rel="noopener noreferrer"
            variante="secondaire"
            className={BOUTON_SECONDAIRE}
          >
            <BrandIcon name="github" size={16} />
            Voir le code source sur GitHub
          </ButtonLink>
        </LegalSection>

        <LegalSection titre="Usage non commercial">
          <P>
            Pour les usages autorisés non commerciaux, Candilog est mis à disposition sous la
            PolyForm Noncommercial License 1.0.0.
          </P>
          <P>
            Cette licence définit précisément les droits accordés et les conditions applicables. Son
            texte officiel constitue la référence.
          </P>
          <ButtonLink
            href={LICENCE_POLYFORM}
            target="_blank"
            rel="noopener noreferrer"
            variante="secondaire"
            className={BOUTON_SECONDAIRE}
          >
            <Icon name="gavel" size={16} />
            Consulter la licence complète
          </ButtonLink>
        </LegalSection>

        <LegalSection titre="Usage commercial">
          <P>Les droits d&apos;utilisation commerciale ne sont pas accordés par défaut.</P>
          <P>
            Toute personne ou organisation souhaitant utiliser Candilog dans un cadre commercial
            nécessitant des droits qui ne sont pas accordés par la licence non commerciale doit
            obtenir une licence commerciale séparée auprès du titulaire des droits.
          </P>
          <CarteInfo titre="Contact pour une licence commerciale">
            <p className="mt-2 text-[14px] font-semibold text-ink">Alexandre Bouttier</p>
            <a
              href={`mailto:${CONTACT_EMAIL}`}
              className="mt-1 inline-flex items-center gap-2 text-[14px]"
            >
              <Icon name="mail" size={17} />
              {CONTACT_EMAIL}
            </a>
          </CarteInfo>
        </LegalSection>

        <LegalSection titre="Quand une licence commerciale peut-elle être nécessaire ?">
          <P>
            Une licence commerciale peut notamment être nécessaire pour commercialiser Candilog,
            distribuer une version dans le cadre d&apos;une offre commerciale, intégrer Candilog à un
            produit ou service commercial ou exploiter commercialement une version dérivée.
          </P>
          <P>
            Cette liste est fournie à titre informatif et ne remplace pas le texte de la licence
            applicable.
          </P>
          <P>
            En cas de doute sur votre usage, contactez le titulaire des droits avant toute
            exploitation commerciale.
          </P>
        </LegalSection>

        <LegalSection titre="Contributions">
          <P>
            Les contributions au projet sont les bienvenues. Les règles applicables aux
            contributions et à leur licence sont décrites directement dans le dépôt Candilog.
          </P>
          <ButtonLink
            href={GITHUB_REPO}
            target="_blank"
            rel="noopener noreferrer"
            variante="secondaire"
            className={BOUTON_SECONDAIRE}
          >
            <BrandIcon name="github" size={16} />
            Contribuer sur GitHub
          </ButtonLink>
        </LegalSection>

        <LegalSection titre="Copyright">
          <P>Copyright © 2026 Alexandre Bouttier</P>
          <P>
            Cette page explique le modèle de licence de Candilog. Elle ne remplace pas le texte
            officiel de la PolyForm Noncommercial License 1.0.0, qui seul fait foi.
          </P>
        </LegalSection>
      </LegalCorps>
    </LegalLayout>
  );
}
