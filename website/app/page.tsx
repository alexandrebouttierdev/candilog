import Image from "next/image";
import {
  ArrowRight,
  BarChart3,
  BriefcaseBusiness,
  CalendarDays,
  Check,
  Download,
  FileText,
  Network,
  ShieldCheck,
  Sparkles,
} from "lucide-react";
import type { LucideIcon } from "lucide-react";
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
        "Application de suivi de candidatures avec gestion des CV, lettres, contacts, entretiens, relances et assistance par intelligence artificielle.",
      applicationCategory: "BusinessApplication",
      operatingSystem: "Windows, macOS, Ubuntu, Fedora",
      inLanguage: "fr-FR",
      featureList: [
        "Suivi Kanban des candidatures",
        "Calendrier des entretiens et relances",
        "Création et analyse ATS des CV",
        "Rédaction de lettres de motivation",
        "Gestion des entreprises et contacts",
        "Statistiques de recherche d’emploi",
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

type Feature = {
  icon: LucideIcon;
  number: string;
  title: string;
  body: string;
  details: string[];
};

const features: Feature[] = [
  {
    icon: BriefcaseBusiness,
    number: "01",
    title: "Suivez chaque candidature",
    body: "Passez du tableau Kanban à la liste détaillée et retrouvez immédiatement la prochaine action.",
    details: ["Filtres et recherche", "Export CSV", "Historique complet"],
  },
  {
    icon: CalendarDays,
    number: "02",
    title: "Ne ratez plus une relance",
    body: "Entretiens, relances et échéances restent liés à la bonne entreprise et au bon poste.",
    details: ["Calendrier", "Rappels", "Comptes rendus"],
  },
  {
    icon: FileText,
    number: "03",
    title: "Adaptez vos documents",
    body: "Créez plusieurs versions de CV et de lettres, comparez-les puis exportez le résultat en PDF.",
    details: ["Versions de CV", "Lettres", "Export PDF"],
  },
  {
    icon: Sparkles,
    number: "04",
    title: "Travaillez avec l’IA",
    body: "Analysez une offre, ciblez un CV et retravaillez une lettre sans quitter votre dossier.",
    details: ["Score ATS", "Réécriture", "7 fournisseurs"],
  },
  {
    icon: Network,
    number: "05",
    title: "Gardez le contexte humain",
    body: "Centralisez entreprises, recruteurs et échanges autour de vos opportunités.",
    details: ["Entreprises", "Contacts", "Liens directs"],
  },
  {
    icon: BarChart3,
    number: "06",
    title: "Comprenez ce qui fonctionne",
    body: "Visualisez votre rythme, vos conversions et l’évolution de vos scores pour ajuster votre recherche.",
    details: ["Entonnoir", "Activité", "Historique ATS"],
  },
];

const aiActions = [
  {
    title: "Lire une offre",
    body: "Le poste, les compétences et les mots-clés importants sont structurés en quelques secondes.",
  },
  {
    title: "Cibler un CV",
    body: "Candilog propose des adaptations et explique les changements avant que vous les validiez.",
  },
  {
    title: "Analyser un PDF",
    body: "Importez un CV existant pour identifier les forces, les manques et le score ATS.",
  },
  {
    title: "Écrire une lettre",
    body: "Générez une première version, puis retravaillez le ton ou un passage avec une consigne simple.",
  },
  {
    title: "Préparer la suite",
    body: "Importez votre profil et transformez vos notes d’entretien en pistes d’amélioration concrètes.",
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

const downloads = [
  {
    os: "Windows",
    detail: "Installateur Windows",
    format: "EXE · x64",
    icon: "/platforms/windows.svg",
    href: `${releaseBase}/candilog-windows-latest.exe`,
  },
  {
    os: "macOS",
    detail: "Image disque macOS",
    format: "DMG",
    icon: "/platforms/apple.svg",
    href: `${releaseBase}/candilog-macos-latest.dmg`,
  },
  {
    os: "Ubuntu",
    detail: "Ubuntu et dérivées",
    format: "AppImage",
    icon: "/platforms/ubuntu.svg",
    href: `${releaseBase}/candilog-ubuntu-latest.AppImage`,
  },
  {
    os: "Fedora",
    detail: "Fedora et dérivées",
    format: "RPM",
    icon: "/platforms/fedora.svg",
    href: `${releaseBase}/candilog-fedora-latest.rpm`,
  },
];

const previews = [
  {
    src: "/screenshots/candidatures.webp",
    eyebrow: "Suivi",
    title: "Votre pipeline en un regard",
    alt: "Candidatures fictives affichées en vue Kanban dans Candilog",
  },
  {
    src: "/screenshots/lettre.webp",
    eyebrow: "Documents",
    title: "Une lettre que vous pouvez vraiment retravailler",
    alt: "Génération d’une lettre avec des données de démonstration fictives dans Candilog",
  },
  {
    src: "/screenshots/statistiques.webp",
    eyebrow: "Analyse",
    title: "Des progrès visibles, pas des impressions",
    alt: "Statistiques fictives de recherche d’emploi dans Candilog",
  },
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
            <Image src="/brand/candilog.png" alt="" width={34} height={34} priority />
            <span>Candilog</span>
          </a>
          <nav aria-label="Navigation principale">
            <a href="#fonctionnalites">Fonctionnalités</a>
            <a href="#intelligence">IA</a>
            <a href="#apercu">Aperçu</a>
          </nav>
          <a className="nav-download" href="#telecharger">
            Télécharger
            <ArrowRight aria-hidden="true" size={16} />
          </a>
        </div>
      </header>

      <main id="main">
        <section className="hero" id="top">
          <div className="page-width hero-copy">
            <p className="eyebrow"><span /> Suivi de candidatures + IA</p>
            <h1>Votre recherche d’emploi.<br />Enfin au même endroit.</h1>
            <p className="hero-lead">
              Candilog réunit candidatures, CV, lettres, contacts et entretiens dans une application claire — avec une IA utile quand vous en avez besoin.
            </p>
            <div className="hero-actions">
              <DownloadButton />
              <a className="action action-secondary" href="#apercu">
                Voir l’application
                <ArrowRight aria-hidden="true" size={18} />
              </a>
            </div>
            <div className="platform-note">
              <span>Windows</span><i />
              <span>macOS</span><i />
              <span>Ubuntu</span><i />
              <span>Fedora</span>
            </div>
          </div>

          <div className="page-width hero-product">
            <div className="journey" aria-label="Parcours géré dans Candilog">
              <span>Offre</span><b />
              <span>Candidature</span><b />
              <span>Documents</span><b />
              <span>Entretien</span><b />
              <span>Relance</span>
            </div>
            <div className="app-frame">
              <div className="window-bar" aria-hidden="true">
                <span /><span /><span />
                <p>Candilog</p>
              </div>
              <Image
                src="/screenshots/dashboard.webp"
                alt="Tableau de bord Candilog rempli avec des données fictives"
                width={2304}
                height={1236}
                priority
                unoptimized
                sizes="(max-width: 1280px) 94vw, 1180px"
              />
            </div>
          </div>
        </section>

        <section className="features-section" id="fonctionnalites">
          <div className="page-width">
            <div className="section-heading">
              <p className="eyebrow"><span /> Le parcours complet</p>
              <h2>Tout ce qu’il faut.<br />Rien qui vous ralentit.</h2>
              <p>Chaque information saisie à un endroit reste disponible dans le reste de votre recherche.</p>
            </div>
            <div className="feature-grid">
              {features.map(({ icon: Icon, ...feature }) => (
                <article className="feature-card" key={feature.number}>
                  <div className="feature-top">
                    <Icon aria-hidden="true" size={23} strokeWidth={1.8} />
                    <span>{feature.number}</span>
                  </div>
                  <h3>{feature.title}</h3>
                  <p>{feature.body}</p>
                  <ul>
                    {feature.details.map((detail) => <li key={detail}>{detail}</li>)}
                  </ul>
                </article>
              ))}
            </div>
            <div className="feature-more">
              <Check aria-hidden="true" size={18} />
              <p><strong>Également inclus</strong> Profil professionnel structuré, import depuis un CV, bibliothèques de versions, sauvegarde, restauration et mises à jour depuis l’application.</p>
            </div>
          </div>
        </section>

        <section className="ai-section" id="intelligence">
          <div className="page-width ai-layout">
            <div className="ai-visual">
              <div className="visual-label">
                <Sparkles aria-hidden="true" size={16} />
                <span>CV ciblé · Offre analysée</span>
              </div>
              <Image
                src="/screenshots/cv-generator.webp"
                alt="Création d’un CV ciblé avec l’IA dans Candilog et des données fictives"
                width={2304}
                height={1236}
                unoptimized
                sizes="(max-width: 900px) 94vw, 58vw"
              />
            </div>
            <div className="ai-copy">
              <p className="eyebrow light"><span /> Intelligence intégrée</p>
              <h2>Une IA qui travaille sur votre dossier.</h2>
              <p className="ai-intro">Pas un chatbot à côté. Candilog utilise votre profil, l’offre et vos documents pour vous aider à produire une candidature plus juste.</p>
              <ol className="ai-actions">
                {aiActions.map((action, index) => (
                  <li key={action.title}>
                    <span>{String(index + 1).padStart(2, "0")}</span>
                    <div><h3>{action.title}</h3><p>{action.body}</p></div>
                  </li>
                ))}
              </ol>
              <div className="provider-row" aria-label="Fournisseurs d’intelligence artificielle compatibles">
                {providers.map((provider) => (
                  <div className="provider" key={provider.name} title={provider.name}>
                    <Image src={provider.icon} alt={provider.name} width={24} height={24} />
                  </div>
                ))}
                <span>+ compatible OpenAI</span>
              </div>
              <p className="local-ai"><ShieldCheck aria-hidden="true" size={17} /> Ollama peut fonctionner localement, sans envoyer vos documents à un service distant.</p>
            </div>
          </div>
        </section>

        <section className="preview-section" id="apercu">
          <div className="page-width">
            <div className="section-heading preview-heading">
              <p className="eyebrow"><span /> Dans l’application</p>
              <h2>Une vue claire à chaque étape.</h2>
              <p>Les images ci-dessous proviennent de l’application avec une base de démonstration entièrement fictive.</p>
            </div>
            <div className="preview-grid">
              {previews.map((preview) => (
                <article className="preview-card" key={preview.src}>
                  <div className="preview-image">
                    <Image src={preview.src} alt={preview.alt} width={2304} height={1236} unoptimized sizes="(max-width: 900px) 94vw, 40vw" />
                  </div>
                  <div className="preview-caption">
                    <span>{preview.eyebrow}</span>
                    <h3>{preview.title}</h3>
                  </div>
                </article>
              ))}
            </div>
          </div>
        </section>

        <section className="privacy-section">
          <div className="page-width privacy-layout">
            <div className="privacy-mark"><ShieldCheck aria-hidden="true" size={36} /></div>
            <div>
              <p className="eyebrow"><span /> Vos données</p>
              <h2>Votre recherche reste la vôtre.</h2>
            </div>
            <div className="privacy-points">
              <p><Check aria-hidden="true" size={17} /> Données conservées sur votre ordinateur</p>
              <p><Check aria-hidden="true" size={17} /> Sauvegarde et restauration complètes</p>
              <p><Check aria-hidden="true" size={17} /> Fournisseur IA choisi par vous</p>
            </div>
          </div>
        </section>

        <section className="download-section" id="telecharger">
          <div className="page-width">
            <div className="download-heading">
              <p className="eyebrow light"><span /> Téléchargement</p>
              <h2>Prêt à reprendre le contrôle&nbsp;?</h2>
              <p>Choisissez votre système. Candilog s’installe directement sur votre ordinateur.</p>
            </div>
            <div className="download-grid">
              {downloads.map((item) => (
                <a className="download-card" href={item.href} key={item.os}>
                  <div className="os-icon">
                    <Image src={item.icon} alt={`Logo ${item.os}`} width={36} height={36} />
                  </div>
                  <div className="download-copy">
                    <h3>{item.os}</h3>
                    <p>{item.detail}</p>
                  </div>
                  <span className="download-format">{item.format}</span>
                  <Download aria-hidden="true" size={20} />
                </a>
              ))}
            </div>
          </div>
        </section>
      </main>

      <footer>
        <div className="page-width footer-layout">
          <a className="brand footer-brand" href="#top">
            <Image src="/brand/candilog.png" alt="" width={26} height={26} />
            <span>Candilog</span>
          </a>
          <p className="footer-author">
            Créé par
            <a href="https://www.alexandrebouttier.fr" target="_blank" rel="noreferrer">
              Alexandre Bouttier <ArrowRight aria-hidden="true" size={12} />
            </a>
          </p>
        </div>
      </footer>
    </div>
  );
}
