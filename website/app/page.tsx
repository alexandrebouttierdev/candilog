import { SiteFooter } from "@/components/layout/SiteFooter";
import { SiteHeader } from "@/components/layout/SiteHeader";
import { AiProviders } from "@/components/landing/AiProviders";
import { AtsPanels } from "@/components/landing/AtsPanels";
import { DownloadCta } from "@/components/landing/DownloadCta";
import { Faq } from "@/components/landing/Faq";
import { Hero } from "@/components/landing/Hero";
import { JourneyTabs } from "@/components/landing/JourneyTabs";
import { Positioning } from "@/components/landing/Positioning";
import { SourceAvailable } from "@/components/landing/SourceAvailable";
import { TrackingBoard } from "@/components/landing/TrackingBoard";

export default function Page() {
  return (
    <>
      <SiteHeader />
      <main>
        <Hero />
        <Positioning />
        <JourneyTabs />
        <AtsPanels />
        <TrackingBoard />
        <AiProviders />
        <SourceAvailable />
        <Faq />
        <DownloadCta />
      </main>
      <SiteFooter />
    </>
  );
}
