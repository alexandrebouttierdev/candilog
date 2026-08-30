import type { Metadata } from "next";

import { LegalLayout } from "@/components/layout/LegalLayout";
import {
  LegalCorps,
  LegalHero,
  LegalSection,
  LigneContact,
  P,
  Sommaire,
  TableauFournisseurs,
} from "@/components/legal/primitives";
import { CONTACT_EMAIL } from "@/lib/data/liens";
import { MISE_A_JOUR_CONFIDENTIALITE } from "@/lib/data/legal";

export const metadata: Metadata = {
  title: "Politique de confidentialité — Candilog",
  description:
    "Où sont enregistrées vos informations et dans quelles situations certaines données peuvent être traitées par des services externes.",
};

const SOMMAIRE = [
  { href: "#local", libelle: "1. Vos données principales sont stockées localement" },
  { href: "#documents", libelle: "2. Documents" },
  { href: "#externes", libelle: "3. Fonctionnalités utilisant des services externes" },
  { href: "#site", libelle: "4. Site internet" },
  { href: "#cookies", libelle: "5. Cookies et traceurs" },
  { href: "#technique", libelle: "6. Données techniques de l'hébergement" },
  { href: "#droits", libelle: "7. Vos droits" },
  { href: "#securite", libelle: "8. Sécurité" },
  { href: "#evolution", libelle: "9. Évolution de cette politique" },
] as const;

const FOURNISSEURS = [
  { nom: "OpenAI", detail: "Fonctions assistées par IA, si vous connectez ce fournisseur avec votre propre clé." },
  { nom: "Anthropic", detail: "Fonctions assistées par IA, si vous connectez ce fournisseur avec votre propre clé." },
  { nom: "Google Gemini", detail: "Fonctions assistées par IA, si vous connectez ce fournisseur avec votre propre clé." },
  { nom: "Mistral AI", detail: "Fonctions assistées par IA, si vous connectez ce fournisseur avec votre propre clé." },
  { nom: "Ollama", detail: "Modèle exécuté localement sur votre appareil : ce mode ne nécessite pas de transmettre vos informations à un service externe." },
] as const;

export default function Page() {
  return (
    <LegalLayout courante="confidentialite">
      <LegalHero
        surTitre="Vos données"
        titre="Politique de confidentialité"
        chapo="Candilog a été conçu pour limiter la centralisation de vos données. Cette page explique où sont enregistrées vos informations et dans quelles situations certaines données peuvent être traitées par des services externes."
        miseAJour={MISE_A_JOUR_CONFIDENTIALITE}
      />

      <Sommaire entrees={SOMMAIRE} />

      <LegalCorps>
        <LegalSection id="local" titre="Vos données principales sont stockées localement" premiere>
          <P>
            Les informations que vous enregistrez dans Candilog, telles que vos candidatures,
            offres, entreprises, entretiens, informations de profil et autres données liées à votre
            recherche d&apos;emploi, sont enregistrées localement sur votre appareil.
          </P>
          <P>
            Candilog n&apos;utilise pas de base de données distante destinée à centraliser ces
            informations pour le compte de l&apos;éditeur du logiciel.
          </P>
          <P>
            Le stockage local vous permet de conserver le contrôle de vos données principales. Il
            implique également que la protection de votre appareil et la sauvegarde de vos fichiers
            restent importantes.
          </P>
        </LegalSection>

        <LegalSection id="documents" titre="Documents">
          <P>
            Les documents que vous importez ou créez dans Candilog sont gérés conformément au
            fonctionnement de l&apos;application.
          </P>
          <P>
            Lorsque certaines fonctionnalités nécessitent un traitement externe, les informations
            nécessaires au fonctionnement de la fonctionnalité concernée peuvent être transmises au
            service utilisé.
          </P>
        </LegalSection>

        <LegalSection id="externes" titre="Fonctionnalités utilisant des services externes">
          <P>
            Certaines fonctionnalités de Candilog peuvent s&apos;appuyer sur des services externes,
            notamment pour proposer des fonctions assistées par intelligence artificielle.
          </P>
          <P>
            Lorsque vous utilisez volontairement une fonctionnalité nécessitant un tel service,
            certaines informations utiles à son fonctionnement peuvent être transmises au
            fournisseur concerné afin de traiter votre demande.
          </P>
          <P>Ces traitements sont distincts du stockage local principal de Candilog.</P>
          <TableauFournisseurs
            entete="Fournisseurs pouvant être configurés dans l'application"
            lignes={FOURNISSEURS}
            note="Les catégories d'informations transmises dépendent de la fonctionnalité utilisée : selon le cas, le texte d'une offre, le contenu d'un document que vous soumettez à l'analyse ou les éléments de votre profil nécessaires à la demande. Les conditions et politiques de confidentialité de chaque fournisseur s'appliquent à ces traitements. Le choix du fournisseur, la fourniture de la clé et le déclenchement de chaque traitement restent à votre initiative."
          />
        </LegalSection>

        <LegalSection id="site" titre="Site internet">
          <P>
            La simple consultation du site officiel de Candilog ne donne pas à l&apos;éditeur accès
            aux données enregistrées localement dans votre application.
          </P>
          <P>
            Les polices et les icônes utilisées par le site sont servies depuis le site lui-même :
            leur affichage n&apos;entraîne pas de requête vers un service tiers. Aucun outil de
            mesure d&apos;audience ni formulaire de collecte n&apos;est utilisé sur le site. Les
            liens vers des services externes, comme le dépôt du code source ou le texte de la
            licence, ne sont suivis que si vous les activez.
          </P>
          <P>
            Le site est hébergé par GitHub Pages (GitHub, Inc.). À ce titre, l&apos;infrastructure de
            l&apos;hébergeur peut traiter des informations techniques liées à la diffusion des pages,
            dans les conditions prévues par sa propre politique de confidentialité.
          </P>
        </LegalSection>

        <LegalSection id="cookies" titre="Cookies et traceurs">
          <P>
            Candilog n&apos;utilise pas de cookies publicitaires ni de mécanisme destiné à suivre
            votre navigation à des fins de profilage.
          </P>
          <P>
            Lorsque des éléments techniques strictement nécessaires au fonctionnement du site sont
            utilisés, ils sont limités à leur finalité technique.
          </P>
        </LegalSection>

        <LegalSection id="technique" titre="Données techniques de l'hébergement">
          <P>
            Comme pour la majorité des sites internet, l&apos;infrastructure d&apos;hébergement peut
            traiter certaines informations techniques nécessaires au fonctionnement et à la sécurité
            du service, telles que l&apos;adresse IP, la date et l&apos;heure d&apos;une requête ou
            des informations techniques relatives au navigateur.
          </P>
          <P>
            Ces traitements techniques relèvent de l&apos;hébergeur du site, GitHub, Inc. Les
            modalités et durées de conservation applicables sont celles définies par ce prestataire
            dans sa politique de confidentialité.
          </P>
        </LegalSection>

        <LegalSection id="droits" titre="Vos droits">
          <P>
            Les données enregistrées uniquement localement par Candilog sont sous votre contrôle sur
            votre propre appareil.
          </P>
          <P>
            Lorsque l&apos;éditeur traite directement des données personnelles dans le cadre du site,
            d&apos;un échange de support ou d&apos;un service externe, vous pouvez exercer les droits
            applicables prévus par la réglementation relative à la protection des données.
          </P>
          <LigneContact email={CONTACT_EMAIL} />
        </LegalSection>

        <LegalSection id="securite" titre="Sécurité">
          <P>
            Candilog cherche à limiter l&apos;exposition de vos informations en privilégiant leur
            stockage local.
          </P>
          <P>
            La sécurité de vos données dépend également de la protection de votre ordinateur, de
            votre système, de vos sauvegardes et, lorsque vous choisissez de les utiliser, des
            services externes nécessaires à certaines fonctionnalités.
          </P>
        </LegalSection>

        <LegalSection id="evolution" titre="Évolution de cette politique">
          <P>
            Cette politique de confidentialité peut être mise à jour afin de refléter
            l&apos;évolution de Candilog, de ses fonctionnalités ou des services utilisés.
          </P>
        </LegalSection>
      </LegalCorps>
    </LegalLayout>
  );
}
