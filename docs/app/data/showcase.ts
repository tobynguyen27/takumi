import type { LucideIcon } from "lucide-react";
import { LayoutTemplate, Type, Zap } from "lucide-react";

// Add your showcase projects here!
export const showcaseProjects: Project[] = [
  {
    title: "Dcard",
    image: "https://fbthumb.dcard.tw/post/260376394",
    url: "https://dcard.tw",
    width: 1200,
    height: 630,
  },
  {
    title: "fuma-nama/fumadocs",
    image: "https://www.fumadocs.dev/og/image.webp",
    url: "https://github.com/fuma-nama/fumadocs",
    width: 1200,
    height: 630,
  },
  {
    title: "kane50613/image-bench",
    image:
      "https://image-bench.kane.tw/render?provider=takumi-webp&template=docs&width=800&height=400",
    url: "https://github.com/kane50613/image-bench",
    width: 800,
    height: 400,
  },
];

export const showcaseTemplates: Template[] = [
  {
    title: "Blog Post",
    image: "/templates/previews/blog-post-template.webp",
    href: "/docs/templates#blog-post-template",
    color: "from-orange-500/20 to-red-500/20",
  },
  {
    title: "Documentation",
    image: "/templates/previews/docs-template.webp",
    href: "/docs/templates#docs-template",
    color: "from-blue-500/20 to-cyan-500/20",
  },
  {
    title: "Product Card",
    image: "/templates/previews/product-card-template.webp",
    href: "/docs/templates#product-card-template",
    color: "from-green-500/20 to-emerald-500/20",
  },
];

export const showcaseFeatures: Feature[] = [
  {
    title: "Advanced Typography",
    description:
      "Full support for variable fonts, RTL, and complex text decorations like skip-ink. Perfectly legible at any scale.",
    icon: Type,
  },
  {
    title: "Native Layout Engine",
    description:
      "A custom Rust-based Flexbox implementation that mirrors browser behavior with zero dependencies.",
    icon: LayoutTemplate,
  },
  {
    title: "Tailwind CSS First",
    description:
      "Use the tools you love. Takumi natively understands Tailwind utility classes for rapid development.",
    icon: Zap,
  },
];

export interface Project {
  title: string;
  image: string;
  url: string;
  width: number;
  height: number;
}

export interface Template {
  title: string;
  image: string;
  href: string;
  color: string;
}

export interface Feature {
  title: string;
  description: string;
  icon: LucideIcon;
}
