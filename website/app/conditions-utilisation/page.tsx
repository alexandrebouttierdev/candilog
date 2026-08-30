import type { Metadata } from "next";
import Link from "next/link";

import { LegalLayout } from "@/components/layout/LegalLayout";
import {
  LegalCorps,
  LegalHero,
  LegalSection,
  LigneContact,
  P,
  Sommaire,
} from "@/components/legal/primitives";
import { ButtonLink } from "@/components/ui/Button";
import { Icon } from "@/components/ui/Icon";
import { CONTACT_EMAIL } from "@/lib/data/liens";
import { MISE_A_JOUR_CONDITIONS } from "@/lib/data/legal";

export const metadata: Metadata = {
  title: "Conditions d'utilisation — Candilog",
  description:
    "Les règles générales applicables à l'utilisation de Candilog et de son site officiel.",
};

const SOMMAIRE = [
  { href: "#objet", libelle: "1. Objet" },
  { href: "#utilisation", libelle: "2. Utilisation de Candilog" },
  { href: "#contenus", libelle: "3. CV, lettres et contenus" },
  { href: "#ats", libelle: "4. Analyse ATS" },
  { href: "#ia", libelle: "5. Intelligence artificielle" },
  { href: "#locales", libelle: "6. Données locales" },
  { href: "#tiers", libelle: "7. Services externes" },
  { href: "#disponibilite", libelle: "8. Disponibilité et évolution" },
  { href: "#sauvegarde", libelle: "9. Sauvegarde et perte de données" },
  { href: "#licence", libelle: "10. Licence" },
  { href: "#pi", libelle: "11. Propriété intellectuelle" },
  { href: "#modifications", libelle: "12. Modifications des conditions" },
  { href: "#contact", libelle: "13. Contact" },
] as const;

export default function Page() {
  return (
    <LegalLayout courante="conditions-utilisation">
      <LegalHero
        surTitre="Conditions"
        titre="Conditions d'utilisation"
        chapo="Les présentes conditions définissent les règles générales applicables à l'utilisation de Candilog et de son site officiel."
        miseAJour={MISE_A_JOUR_CONDITIONS}
      />

      <Sommaire entrees={SOMMAIRE} />

      <LegalCorps>
        <LegalSection id="objet" titre="1. Objet" premiere>
          <P>
            Candilog est une application destinée à aider les utilisateurs à organiser leur
            recherche d&apos;emploi, gérer leurs candidatures et leurs documents et utiliser
            différents outils d&apos;assistance à la préparation de leurs démarches.
          </P>
        </LegalSection>

        <LegalSection id="utilisation" titre="2. Utilisation de Candilog">
          <P>
            L&apos;utilisation de Candilog doit respecter les lois et réglementations applicables
            ainsi que les conditions de licence du logiciel.
          </P>
          <P>
            L&apos;utilisateur reste responsable des informations, documents et contenus qu&apos;il
            enregistre, importe, modifie, génère ou utilise avec Candilog.
          </P>
        </LegalSection>

        <LegalSection id="contenus" titre="3. CV, lettres et contenus">
          <P>
            Candilog peut proposer des outils destinés à faciliter la préparation ou
            l&apos;adaptation de documents liés à une recherche d&apos;emploi.
          </P>
          <P>
            L&apos;utilisateur doit relire, vérifier et, si nécessaire, corriger tout contenu avant
            de l&apos;utiliser ou de l&apos;envoyer à un recruteur ou à un tiers.
          </P>
          <P>
            Candilog ne garantit pas l&apos;exactitude, l&apos;exhaustivité ou la pertinence de tout
            contenu généré ou suggéré automatiquement.
          </P>
        </LegalSection>

        <LegalSection id="ats" titre="4. Analyse ATS">
          <P>
            Les outils d&apos;analyse ATS proposés par Candilog fournissent des indications destinées
            à aider l&apos;utilisateur à comparer un CV et une offre d&apos;emploi.
          </P>
          <P>
            Les résultats affichés ne reproduisent pas nécessairement le fonctionnement d&apos;un
            système de recrutement particulier et ne constituent aucune garantie qu&apos;un CV sera
            accepté par un logiciel ATS, lu par un recruteur ou qu&apos;une candidature aboutira à un
            entretien ou à une embauche.
          </P>
        </LegalSection>

        <LegalSection id="ia" titre="5. Intelligence artificielle">
          <P>
            Certaines fonctionnalités peuvent s&apos;appuyer sur des systèmes d&apos;intelligence
            artificielle ou d&apos;autres services automatisés.
          </P>
          <P>
            Les réponses, analyses ou suggestions produites par ces systèmes peuvent contenir des
            erreurs, des approximations ou des informations inadaptées au contexte de
            l&apos;utilisateur.
          </P>
          <P>
            L&apos;utilisateur reste responsable de la vérification et de l&apos;utilisation des
            contenus produits avec l&apos;aide de ces fonctionnalités.
          </P>
        </LegalSection>

        <LegalSection id="locales" titre="6. Données locales">
          <P>
            Les données principales de Candilog sont enregistrées localement sur l&apos;appareil de
            l&apos;utilisateur.
          </P>
          <P>
            L&apos;utilisateur est responsable de la protection de son appareil et de la réalisation
            de sauvegardes adaptées de ses données, sauf lorsqu&apos;une fonctionnalité proposée par
            Candilog indique explicitement un fonctionnement différent.
          </P>
        </LegalSection>

        <LegalSection id="tiers" titre="7. Services externes">
          <P>
            Certaines fonctionnalités peuvent nécessiter des services fournis par des tiers,
            notamment les fournisseurs d&apos;intelligence artificielle que l&apos;utilisateur
            choisit de connecter avec sa propre clé.
          </P>
          <P>
            Leur disponibilité, leurs conditions d&apos;utilisation et leurs politiques de
            confidentialité peuvent dépendre de ces fournisseurs.
          </P>
          <P>Candilog ne peut pas garantir la disponibilité permanente d&apos;un service tiers.</P>
          <P>
            Le détail des services concernés est précisé dans la{" "}
            <Link href="/confidentialite#externes">politique de confidentialité</Link>.
          </P>
        </LegalSection>

        <LegalSection id="disponibilite" titre="8. Disponibilité et évolution">
          <P>Candilog est un logiciel en évolution.</P>
          <P>
            Des fonctionnalités peuvent être ajoutées, modifiées, remplacées ou supprimées au fil
            des versions.
          </P>
          <P>
            L&apos;éditeur s&apos;efforce de maintenir le logiciel fonctionnel mais ne garantit pas
            une disponibilité permanente, l&apos;absence totale d&apos;erreurs ni la compatibilité
            avec toutes les configurations matérielles ou logicielles.
          </P>
        </LegalSection>

        <LegalSection id="sauvegarde" titre="9. Sauvegarde et perte de données">
          <P>
            L&apos;utilisateur est invité à conserver des sauvegardes adaptées de ses données et
            documents importants.
          </P>
          <P>
            Dans les limites autorisées par la loi applicable, l&apos;éditeur ne peut être tenu
            responsable d&apos;une perte de données résultant notamment d&apos;une panne, d&apos;une
            suppression accidentelle, d&apos;une mauvaise manipulation, d&apos;un problème matériel
            ou logiciel ou de l&apos;absence de sauvegarde.
          </P>
        </LegalSection>

        <LegalSection id="licence" titre="10. Licence">
          <P>
            L&apos;utilisation, la modification et la distribution de Candilog sont soumises aux
            conditions de licence applicables.
          </P>
          <P>
            Les usages autorisés non commerciaux sont régis par la PolyForm Noncommercial License
            1.0.0.
          </P>
          <P>
            Les droits commerciaux nécessitent une licence commerciale séparée lorsqu&apos;ils ne
            sont pas accordés par la licence applicable.
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

        <LegalSection id="pi" titre="11. Propriété intellectuelle">
          <P>
            Candilog, son identité visuelle, son logo et les autres éléments originaux du projet
            restent protégés par les droits de propriété intellectuelle applicables.
          </P>
          <P>La disponibilité publique du code source n&apos;implique pas l&apos;abandon de ces droits.</P>
          <P>Les éléments appartenant à des tiers restent soumis à leurs propres droits et licences.</P>
        </LegalSection>

        <LegalSection id="modifications" titre="12. Modifications des conditions">
          <P>
            Ces conditions peuvent évoluer afin de refléter les modifications du logiciel, de ses
            fonctionnalités ou du cadre applicable.
          </P>
          <P>
            La version publiée sur le site constitue la version en vigueur à la date indiquée sur
            cette page.
          </P>
        </LegalSection>

        <LegalSection id="contact" titre="13. Contact">
          <P>
            Pour toute question concernant ces conditions, utilisez le moyen de contact officiel
            indiqué sur le site Candilog.
          </P>
          <LigneContact email={CONTACT_EMAIL} />
        </LegalSection>
      </LegalCorps>
    </LegalLayout>
  );
}
