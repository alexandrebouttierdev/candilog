import Image from "next/image";
import { ArrowRight, Download, LockKeyhole } from "lucide-react";
import { DownloadButton } from "@/components/download-button";

const releaseBase =
  "https://github.com/alexandrebouttierdev/candilog-releases/releases/latest/download";
const siteUrl = (process.env.NEXT_PUBLIC_SITE_URL ?? "http://localhost:3000").replace(/\/$/, "");

const structuredData = {
  "@context": "https://schema.org",
  "@graph": [
    {
      "@type": "WebSite",
      "@id": `${siteUrl}/#website`,
      name: "Candilog",
      url: `${siteUrl}/`,
      inLanguage: "fr-FR",
      creator: {
        "@type": "Person",
        name: "Alexandre Bouttier",
        url: "https://www.alexandrebouttier.fr",
      },
    },
    {
      "@type": "SoftwareApplication",
      "@id": `${siteUrl}/#application`,
      name: "Candilog",
      url: `${siteUrl}/`,
      image: `${siteUrl}/opengraph-image.png`,
      screenshot: `${siteUrl}/screenshots/dashboard.webp`,
      description:
        "Application de bureau pour organiser les candidatures, les contacts, les rendez-vous et les documents d'une recherche d'emploi.",
      applicationCategory: "BusinessApplication",
      operatingSystem: "Windows, macOS, Ubuntu, Fedora",
      inLanguage: "fr-FR",
      featureList: [
        "Suivi des candidatures",
        "Calendrier des rendez-vous et relances",
        "Gestion des contacts et entreprises",
        "Création de CV ciblés",
        "Rédaction de lettres de motivation",
        "Analyse de la recherche d'emploi",
      ],
      downloadUrl: [
        `${releaseBase}/candilog-windows-latest.exe`,
        `${releaseBase}/candilog-macos-latest.dmg`,
        `${releaseBase}/candilog-ubuntu-latest.AppImage`,
        `${releaseBase}/candilog-fedora-latest.rpm`,
      ],
      author: {
        "@type": "Person",
        name: "Alexandre Bouttier",
        url: "https://www.alexandrebouttier.fr",
      },
    },
  ],
};

const platformDownloads = [
  {
    name: "Windows",
    format: "EXE",
    icon: "/platforms/windows.svg",
    href: `${releaseBase}/candilog-windows-latest.exe`,
  },
  {
    name: "macOS",
    format: "DMG",
    icon: "/platforms/apple.svg",
    href: `${releaseBase}/candilog-macos-latest.dmg`,
  },
  {
    name: "Ubuntu",
    format: "AppImage",
    icon: "/platforms/ubuntu.svg",
    href: `${releaseBase}/candilog-ubuntu-latest.AppImage`,
  },
  {
    name: "Fedora",
    format: "RPM",
    icon: "/platforms/fedora.svg",
    href: `${releaseBase}/candilog-fedora-latest.rpm`,
  },
];

const providers = [
  { name: "Ollama", icon: "/providers/ollama.svg" },
  { name: "Claude", icon: "/providers/claude.svg" },
  { name: "OpenAI", icon: "/providers/openai.svg" },
  { name: "Gemini", icon: "/providers/gemini.svg" },
  { name: "Mistral", icon: "/providers/mistral.svg" },
  { name: "NVIDIA", icon: "/providers/nvidia.svg" },
];

export default function Home() {
  return (
    <div className="site-shell">
      <script
        type="application/ld+json"
        dangerouslySetInnerHTML={{
          __html: JSON.stringify(structuredData).replace(/</g, "\\u003c"),
        }}
      />

      <a className="skip-link" href="#main">
        Aller au contenu
      </a>

      <header className="topbar">
        <div className="page-width nav-shell">
          <a className="brand" href="#top" aria-label="Candilog, accueil">
            <Image src="/brand/candilog.png" alt="" width={34} height={34} />
            <span>Candilog</span>
          </a>

          <nav aria-label="Navigation principale">
            <a href="#apercu">Aperçu</a>
            <a href="#documents">Documents</a>
            <a href="#confidentialite">Confidentialité</a>
          </nav>

          <a className="nav-action" href="#telecharger">
            Installer Candilog
          </a>
        </div>
      </header>

      <main id="main">
        <section className="hero" id="top">
          <div className="page-width hero-copy">
            <h1>Votre recherche d’emploi, sans les onglets ouverts.</h1>
            <p>
              Candilog réunit offres, contacts, documents et rendez-vous dans un espace privé sur votre ordinateur.
            </p>
            <div className="hero-actions">
              <DownloadButton />
              <a className="action action-secondary" href="#apercu">
                Voir le produit
                <ArrowRight aria-hidden="true" size={18} strokeWidth={1.8} />
              </a>
            </div>
          </div>

          <div className="hero-stage" aria-label="Aperçu de Candilog">
            <div className="hero-window">
              <Image
                src="/screenshots/dashboard.webp"
                alt="Tableau de bord Candilog avec des données de démonstration fictives"
                width={2304}
                height={1236}
                sizes="(max-width: 720px) 900px, 1500px"
                preload
              />
            </div>
          </div>
        </section>

        <aside className="fact-line" aria-label="Informations essentielles">
          <div className="page-width fact-line-inner">
            <p>Application de bureau</p>
            <p>Windows, macOS et Linux</p>
            <p>Données conservées localement</p>
          </div>
        </aside>

        <section className="manifesto page-width">
          <p className="section-kicker">Le dossier complet</p>
          <h2>Une candidature ne tient pas dans une ligne de tableur.</h2>
          <p>
            Il y a l’offre, le contact, le rendez-vous, la bonne version du CV et ce qu’il faut faire ensuite. Candilog garde le lien.
          </p>
        </section>

        <section className="product-story page-width" id="apercu">
          <div className="story-index" aria-hidden="true">
            <span>Suivi</span>
            <span>Agenda</span>
            <span>Analyse</span>
          </div>

          <div className="story-flow">
            <figure className="story-chapter chapter-wide">
              <div className="product-frame">
                <Image
                  src="/screenshots/candidatures.webp"
                  alt="Candidatures fictives organisées par étape dans Candilog"
                  width={2304}
                  height={1236}
                  sizes="(max-width: 820px) 100vw, 1040px"
                />
              </div>
              <figcaption>
                <strong>Le fil ne se perd plus.</strong>
                <span>Chaque dossier montre son état et la prochaine action utile.</span>
              </figcaption>
            </figure>

            <figure className="story-chapter chapter-offset">
              <div className="product-frame">
                <Image
                  src="/screenshots/calendrier.webp"
                  alt="Calendrier Candilog avec des rendez-vous et relances fictifs"
                  width={2304}
                  height={1236}
                  sizes="(max-width: 820px) 100vw, 840px"
                />
              </div>
              <figcaption>
                <strong>Le prochain mouvement est déjà là.</strong>
                <span>Entretiens et relances prennent place dans le même agenda.</span>
              </figcaption>
            </figure>

            <figure className="story-chapter chapter-quiet">
              <div className="product-frame">
                <Image
                  src="/screenshots/statistiques.webp"
                  alt="Analyse de candidatures fictives dans Candilog"
                  width={2304}
                  height={1236}
                  sizes="(max-width: 820px) 100vw, 920px"
                />
              </div>
              <figcaption>
                <strong>Le recul devient concret.</strong>
                <span>Les réponses et les délais montrent où concentrer l’effort.</span>
              </figcaption>
            </figure>
          </div>
        </section>

        <section className="documents" id="documents">
          <div className="page-width documents-layout">
            <div className="documents-visual">
              <div className="product-frame document-frame">
                <Image
                  src="/screenshots/cv-generator.webp"
                  alt="Création d’un CV ciblé pour une offre fictive dans Candilog"
                  width={2304}
                  height={1236}
                  sizes="(max-width: 900px) 100vw, 760px"
                />
              </div>
            </div>

            <div className="documents-copy">
              <h2>Repartez de vos faits, pas d’une page blanche.</h2>
              <p>
                Candilog lit l’offre, rapproche les compétences et prépare une base que vous pouvez relire, corriger ou refuser.
              </p>
              <p className="provider-intro">Vous choisissez le moteur.</p>
              <div className="provider-row" aria-label="Moteurs d’intelligence artificielle compatibles">
                {providers.map((provider) => (
                  <Image
                    key={provider.name}
                    src={provider.icon}
                    alt={provider.name}
                    width={26}
                    height={26}
                  />
                ))}
              </div>
              <p className="local-ai">Ollama permet de travailler avec un modèle installé localement.</p>
            </div>
          </div>
        </section>

        <section className="privacy page-width" id="confidentialite">
          <div className="privacy-heading">
            <LockKeyhole aria-hidden="true" size={24} strokeWidth={1.6} />
            <h2>Votre dossier reste un dossier privé.</h2>
            <p>
              Les candidatures sont enregistrées dans une base locale. Aucun compte Candilog n’est nécessaire pour les retrouver.
            </p>
          </div>

          <div className="privacy-details">
            <article>
              <span>Stockage</span>
              <strong>Sur votre ordinateur</strong>
              <p>Une base SQLite réunit votre historique et vos documents.</p>
            </article>
            <article>
              <span>Sauvegarde</span>
              <strong>À votre initiative</strong>
              <p>Vous exportez et restaurez votre espace quand vous le souhaitez.</p>
            </article>
            <article>
              <span>Assistant</span>
              <strong>Avec votre modèle</strong>
              <p>Local avec Ollama, ou connecté au fournisseur que vous utilisez déjà.</p>
            </article>
          </div>
        </section>

        <section className="download-section" id="telecharger">
          <div className="page-width download-layout">
            <div className="download-copy">
              <h2>Installez votre espace de travail.</h2>
              <p>Choisissez votre système. Vos dossiers pourront commencer au même endroit.</p>
              <DownloadButton />
            </div>

            <div className="platform-list" aria-label="Téléchargements par système">
              {platformDownloads.map((platform) => (
                <a key={platform.name} href={platform.href} aria-label={`Télécharger Candilog pour ${platform.name}`}>
                  <Image src={platform.icon} alt="" width={28} height={28} />
                  <span>{platform.name}</span>
                  <small>{platform.format}</small>
                  <Download aria-hidden="true" size={18} strokeWidth={1.7} />
                </a>
              ))}
            </div>
          </div>
        </section>
      </main>

      <footer>
        <div className="page-width footer-inner">
          <a className="brand footer-brand" href="#top" aria-label="Candilog, retour en haut">
            <Image src="/brand/candilog.png" alt="" width={30} height={30} />
            <span>Candilog</span>
          </a>
          <p>Une application de bureau conçue par Alexandre Bouttier.</p>
          <a href="https://github.com/alexandrebouttierdev/candilog-releases">Versions publiées</a>
        </div>
      </footer>
    </div>
  );
}
