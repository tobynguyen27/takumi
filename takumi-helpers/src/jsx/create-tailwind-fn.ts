// Modified from https://github.com/vercel/satori/blob/2a0878a7f329bdba3a17ad68f71186a47add0dde/src/handler/tailwind.ts

import type { TwConfig } from "twrnc";
import { create } from "twrnc/create";

type TwPlugin = NonNullable<TwConfig["plugins"]>[number];

declare module "react" {
  // biome-ignore lint/correctness/noUnusedVariables: The T is used for type inheritance
  interface HTMLAttributes<T> {
    tw?: string;
  }
}

const defaultShadows: TwPlugin = {
  handler({ addUtilities }) {
    addUtilities({
      "shadow-sm": { boxShadow: "0 1px 2px 0 rgb(0 0 0 / 0.05)" },
      shadow: {
        boxShadow:
          "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)",
      },
      "shadow-md": {
        boxShadow:
          "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)",
      },
      "shadow-lg": {
        boxShadow:
          "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)",
      },
      "shadow-xl": {
        boxShadow:
          "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)",
      },
      "shadow-2xl": {
        boxShadow: "0 25px 50px -12px rgb(0 0 0 / 0.25)",
      },
      "shadow-inner": {
        boxShadow: "inset 0 2px 4px 0 rgb(0 0 0 / 0.05)",
      },
      "shadow-none": { boxShadow: "0 0 #0000" },
      block: {
        display: "block",
      },
      inline: {
        display: "inline",
      },
      grid: {
        display: "grid",
      },
    });
  },
};

/**
 * @description Creates a function that can be used to parse tailwind classes into inline styles.
 * @param config The tailwind config to use.
 */
export function createTailwindFn(config?: TwConfig) {
  const inner = create(
    {
      ...config,
      plugins: [...(config?.plugins ?? []), defaultShadows],
    },
    "web",
    {
      major: 0,
      minor: 0,
      patch: 0,
    },
  );

  return function tw(...args: Parameters<typeof inner.style>) {
    const styles = inner.style(...args);

    // twrnc converts line height to a pixel value,
    // but by default number means em, so we need to mark it as px explicitly.
    if (typeof styles.lineHeight === "number") {
      styles.lineHeight = `${styles.lineHeight}px`;
    }

    // Since color and shadow definitions are separated,
    // we need to merge them together.
    if (styles.shadowColor && typeof styles.boxShadow === "string") {
      styles.boxShadow = styles.boxShadow.replace(
        /rgba?\([^)]+\)/g,
        styles.shadowColor.toString(),
      );
    }

    return styles;
  };
}
