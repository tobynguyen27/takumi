import { Link2Icon } from "lucide-react";
import { useMemo } from "react";
import type { Project } from "../../data/showcase";
import { GithubIcon } from "./github-icon";

export interface ShowcaseCardProps {
  project: Project;
}

export function ShowcaseCard({ project }: ShowcaseCardProps) {
  const title = useMemo(() => {
    if (!project.title) {
      const { hostname, pathname } = new URL(project.url);

      if (hostname === "github.com") {
        const [owner, repo] = pathname.split("/").filter(Boolean);

        return `${owner}/${repo}`;
      }

      return hostname;
    }

    return project.title;
  }, [project.title, project.url]);

  const icon = useMemo(() => {
    if (project.url.includes("github.com")) {
      return <GithubIcon />;
    }

    return <Link2Icon size={18} />;
  }, [project.url]);

  return (
    <a
      href={project.url}
      target="_blank"
      rel="noopener noreferrer"
      className="border rounded-lg overflow-hidden group bg-muted/10"
    >
      <div className="relative aspect-1200/630 overflow-hidden bg-muted/30">
        <img
          src={project.image}
          alt="Blur background"
          className="absolute inset-0 w-full h-full object-cover blur-xs scale-110 opacity-75 select-none pointer-events-none"
          width={project.width}
          height={project.height}
          loading="lazy"
          decoding="async"
        />
        <img
          src={project.image}
          alt={title}
          className="relative w-full h-full object-contain transition-transform duration-500 group-hover:scale-[1.02]"
          width={project.width}
          height={project.height}
          loading="lazy"
          decoding="async"
        />
      </div>
      <div className="px-4 py-2 border-t flex items-center gap-2 text-foreground/80 group-hover:text-foreground transition-colors duration-300">
        {icon}
        <span className="text-sm font-medium">{title}</span>
      </div>
    </a>
  );
}
