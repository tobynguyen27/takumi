import { HomeLayout } from "fumadocs-ui/layouts/home";
import JSConfetti from "js-confetti";
import { Heart, LayoutTemplate } from "lucide-react";
import { useCallback, useRef } from "react";
import { ShowcaseCard } from "~/components/showcase/showcase-card";
import { TemplateCard } from "~/components/showcase/template-card";
import { baseOptions } from "~/layout-config";
import { showcaseProjects, showcaseTemplates } from "../data/showcase";

export default function Showcase() {
  const confettiRef = useRef<JSConfetti | null>(null);

  const onConfetti = useCallback((e: React.MouseEvent<HTMLButtonElement>) => {
    if (!confettiRef.current) {
      confettiRef.current = new JSConfetti();
    }

    const rect = e.currentTarget.getBoundingClientRect();
    const x = rect.left + rect.width / 2;
    const y = rect.top + rect.height / 2;

    confettiRef.current.addConfettiAtPosition({
      emojis: ["❤️", "🪓"],
      emojiSize: 50,
      confettiNumber: 25,
      confettiDispatchPosition: { x, y },
    });
  }, []);

  return (
    <HomeLayout {...baseOptions}>
      <title>Showcase</title>
      <meta
        name="description"
        content="Discover how developers are using Takumi to power their dynamic image generation."
      />
      <div className="container py-24 px-4 mx-auto max-w-8xl">
        <div className="flex flex-col items-center text-center mb-16">
          <div className="relative mb-6">
            <div className="absolute -inset-4 bg-primary/20 blur-2xl rounded-full animate-pulse duration-300" />
            <button
              type="button"
              onClick={onConfetti}
              className="relative group transition-transform active:scale-95 cursor-pointer outline-none"
            >
              <Heart className="w-16 h-16 text-primary fill-primary drop-shadow-[0_0_15px_rgba(239,68,68,0.5)] transition-transform group-hover:scale-110 duration-300" />
            </button>
          </div>
          <h1 className="text-4xl md:text-6xl font-bold mb-6 tracking-tight">
            Built with <span className="text-primary">Takumi</span>
          </h1>
          <p className="text-muted-foreground text-lg md:text-xl max-w-2xl mx-auto text-pretty">
            Discover how developers are using Takumi to power their dynamic
            image generation.
          </p>
        </div>

        <section className="mb-24 grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
          {showcaseProjects.map((project) => (
            <ShowcaseCard key={project.url} project={project} />
          ))}
        </section>

        <section className="mb-24">
          <h2 className="text-2xl font-semibold mb-8 flex items-center gap-2">
            <LayoutTemplate className="w-6 h-6 text-blue-500" />
            Ready-to-use Templates
          </h2>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6">
            {showcaseTemplates.map((item) => (
              <TemplateCard key={item.title} item={item} />
            ))}
          </div>
        </section>

        <div className="rounded-3xl bg-primary/80 p-8 md:p-16 text-primary-foreground text-center relative overflow-hidden">
          <div className="absolute top-0 left-0 w-full h-full bg-[radial-gradient(circle_at_30%_50%,rgba(255,255,255,0.1),transparent)]" />
          <div className="relative z-10">
            <h2 className="text-3xl md:text-4xl font-bold mb-4">
              Want to be featured?
            </h2>
            <p className="text-primary-foreground/80 mb-8 max-w-xl mx-auto">
              Built something cool with Takumi? We&apos;d love to show it off!
              <br />
              Submit your project to our GitHub repository.
            </p>
            <div className="flex flex-wrap justify-center gap-4">
              <a
                href="https://github.com/kane50613/takumi/edit/master/docs/app/data/showcase.ts"
                target="_blank"
                rel="noreferrer"
                className="bg-white text-primary px-8 py-3 rounded-full font-medium hover:shadow-lg transition-transform hover:-translate-y-1"
              >
                Make a Pull Request
              </a>
            </div>
          </div>
        </div>
      </div>
    </HomeLayout>
  );
}
