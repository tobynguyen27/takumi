export interface StatBlockProps {
  value: string;
  label: string;
  delay: number;
}

export function StatBlock({ value, label, delay }: StatBlockProps) {
  return (
    <div
      className="flex-1 w-full flex flex-col items-center px-10 py-6 max-sm:px-8 max-sm:py-4 bg-background backdrop-blur-xl animate-reveal-up"
      style={{ animationDelay: `${delay}ms` }}
    >
      <span className="font-display text-[2.25rem] font-bold tracking-tight text-foreground">
        {value}
      </span>
      <span className="text-[0.85rem] text-muted-foreground mt-1 whitespace-nowrap">
        {label}
      </span>
    </div>
  );
}
