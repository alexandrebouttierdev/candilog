import type { Metadata } from "next";

import { LegalLayout } from "@/components/layout/LegalLayout";
import {
  LegalCorps,
  LegalHero,
  LegalSection,
  LigneDefinition,
  ListeDefinitions,
  P,
} from "@/components/legal/primitives";
import { ButtonLink } from "@/components/ui/Button";
import { Icon } from "@/components/ui/Icon";
import { CONTACT_EMAIL } from "@/lib/data/liens";

export const metadata: Metadata = {
  title: "Mentions légales — Candilog",
  description:
    "Informations relatives à l'édition et à l'hébergement du site officiel de Candilog.",
};

export default function Page() {
  return (
    <LegalLayout courante="mentions-legales">
      <LegalHero
        surTitre="Informations légales"
        titre="Mentions légales"
        chapo="Les présentes mentions légales précisent les informations relatives à l'édition et à l'hébergement du site officiel de Candilog."
      />

      <LegalCorps>
        <LegalSection titre="Éditeur du site" premiere>
          <ListeDefinitions>
            <LigneDefinition libelle="Éditeur">Alexandre Bouttier</LigneDefinition>
            <LigneDefinition libelle="Contact">
              <a href={`mailto:${CONTACT_EMAIL}`}>{CONTACT_EMAIL}</a>
            </LigneDefinition>
            <LigneDefinition libelle="Responsable de la publication">
              Alexandre Bouttier
            </LigneDefinition>
          </ListeDefinitions>
        </LegalSection>

        <LegalSection titre="Hébergement du site">
          <P>
            Le site officiel de Candilog est hébergé par GitHub Pages, service fourni par
            GitHub, Inc. L&apos;application Candilog, elle, s&apos;exécute sur l&apos;appareil de
            l&apos;utilisateur et n&apos;est pas hébergée sur un serveur.
          </P>
          <div className="mt-[18px] border-t border-line">
            <LigneDefinition libelle="Hébergeur">GitHub, Inc. — GitHub Pages</LigneDefinition>
            <LigneDefinition libelle="Adresse">
              88 Colin P. Kelly Jr. Street, San Francisco, CA 94107, États-Unis
            </LigneDefinition>
            <LigneDefinition libelle="Site">
              <a href="https://github.com" target="_blank" rel="noopener noreferrer">
                github.com
              </a>
            </LigneDefinition>
          </div>
        </LegalSection>

        <LegalSection titre="Propriété intellectuelle">
          <P>
            Le site Candilog, son identité visuelle, son logo, ses textes, ses interfaces et les
            éléments originaux qui le composent sont protégés par les règles applicables en
            matière de propriété intellectuelle.
          </P>
          <P>
            Le code source de Candilog est mis à disposition selon les conditions précisées sur la
            page Licence. La mise à disposition du code source n&apos;emporte aucun transfert de
            propriété intellectuelle au-delà des droits expressément accordés par les licences
            applicables.
          </P>
          <P>
            Les marques, bibliothèques, polices, icônes et autres éléments appartenant à des tiers
            restent soumis aux droits et licences de leurs propriétaires respectifs.
          </P>
        </LegalSection>

        <LegalSection titre="Licence du logiciel">
          <P>Candilog est un logiciel source available.</P>
          <P>
            Les usages autorisés non commerciaux sont régis par la PolyForm Noncommercial License
            1.0.0.
          </P>
          <P>
            Toute utilisation commerciale nécessitant des droits qui ne sont pas accordés par cette
            licence doit faire l&apos;objet d&apos;une licence commerciale séparée.
          </P>
          <ButtonLink
            href="/licence"
            variante="secondaire"
            className="mt-5 h-[34px] px-[14px] text-[13px]"
          >
            <Icon name="gavel" size={16} />
            Consulter la licence
          </ButtonLink>
        </LegalSection>

        <LegalSection titre="Informations publiées">
          <P>
            Les informations présentées sur ce site ont pour objectif de présenter Candilog et son
            fonctionnement. Elles peuvent évoluer à mesure que le logiciel est développé et mis à
            jour.
          </P>
        </LegalSection>
      </LegalCorps>
    </LegalLayout>
  );
}
