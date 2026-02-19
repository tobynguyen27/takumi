export interface TemplateCardProps {
  item: {
    title: string;
    href: string;
    image: string;
  };
}

export function TemplateCard({ item }: TemplateCardProps) {
  return (
    <a href={item.href} className="group flex flex-col space-y-4">
      <div className="relative aspect-1200/630 overflow-hidden rounded-lg bg-muted/30 border">
        <img
          src={item.image}
          alt=""
          className="absolute inset-0 w-full h-full object-cover blur-2xl scale-110 opacity-50 select-none pointer-events-none"
        />
        <img
          src={item.image}
          alt={`${item.title} layout example`}
          width={1200}
          height={630}
          className="relative w-full h-full object-contain transition-transform duration-700 group-hover:scale-105"
        />
      </div>
      <h3 className="font-semibold text-lg inline-flex items-center gap-2">
        {item.title} Template
        <span className="text-primary group-hover:translate-x-1 transition-transform">
          &rarr;
        </span>
      </h3>
    </a>
  );
}
