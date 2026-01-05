import crypto from "node:crypto";
import { existsSync } from "node:fs";
import { mkdir, readdir, writeFile } from "node:fs/promises";
import path from "node:path";
import type { Plugin } from "vite";

function generateHash(url: string): string {
  return crypto.createHash("md5").update(url).digest("hex").slice(0, 8);
}

async function downloadAsset(url: string, filePath: string): Promise<void> {
  console.log(`[remote-assets] Fetching: ${url}`);
  const res = await fetch(url);
  if (!res.ok) throw new Error(`Status ${res.status}`);

  const buffer = Buffer.from(await res.arrayBuffer());
  await writeFile(filePath, buffer);
}

async function findCachedFile(
  outputDir: string,
  hash: string,
): Promise<string | null> {
  if (!existsSync(outputDir)) return null;

  const files = await readdir(outputDir);
  const existingFile = files.find((f) => f.startsWith(hash));
  return existingFile ? path.join(outputDir, existingFile) : null;
}

export function remoteAssets(): Plugin {
  return {
    name: "takumi-remote-assets",
    enforce: "pre",
    async transform(code: string, id: string) {
      if (!id.includes("app/data/showcase.ts")) return;

      const urlRegex = /image:\s*["'](https?:\/\/[^"']+)["']/g;
      const matches = [...code.matchAll(urlRegex)];
      if (matches.length === 0) return;

      const outputDir = path.join(path.dirname(id), ".remote-assets");
      await mkdir(outputDir, { recursive: true });

      let newCode = code;
      const imports: string[] = [];

      for (let i = 0; i < matches.length; i++) {
        const url = matches[i][1];
        const hash = generateHash(url);

        let filePath = await findCachedFile(outputDir, hash);

        if (!filePath) {
          try {
            filePath = path.join(outputDir, hash);
            await downloadAsset(url, filePath);
          } catch (err) {
            console.error(`[remote-assets] Failed to download ${url}:`, err);
            continue;
          }
        }

        const varName = `__remote_asset_${i}`;
        const fileName = path.basename(filePath);

        imports.push(`import ${varName} from "./.remote-assets/${fileName}";`);

        // Escape special regex characters in URL
        const escapedUrl = url.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
        newCode = newCode.replace(
          new RegExp(`["']${escapedUrl}["']`, "g"),
          varName,
        );
      }

      return {
        code: `${imports.join("\n")}\n${newCode}`,
        map: null,
      };
    },
  };
}
