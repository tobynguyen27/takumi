import { reactRouter } from "@react-router/dev/vite";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vite";
import topLevelAwait from "vite-plugin-top-level-await";
import tsconfigPaths from "vite-tsconfig-paths";

export default defineConfig({
  build: {
    rollupOptions: {
      external: ["shiki"],
    },
  },
  ssr: {
    external: ["@takumi-rs/core"],
  },
  plugins: [tailwindcss(), reactRouter(), tsconfigPaths(), topLevelAwait()],
});
