// Purpose: Provides a small CLI for regenerating normalized JSON assets from the organized repository sources.

import path from "node:path";
import { fileURLToPath } from "node:url";

import { ImporterError } from "./errors.js";
import { importRepositoryData } from "./importer.js";
import type { ImportOptions } from "./types.js";

async function main(): Promise<void> {
  const args = parseArgs(process.argv.slice(2));
  const repoRoot = path.resolve(args.root ?? path.join(currentDirectory(), "../../.."));
  const importOptions: ImportOptions = {
    repoRoot,
    writeFiles: true
  };

  if (typeof args.output === "string") {
    importOptions.outputRoot = path.resolve(repoRoot, args.output);
  }

  try {
    const outputs = await importRepositoryData(importOptions);

    process.stdout.write(
      `${JSON.stringify(
        {
          ok: true,
          repoRoot,
          counts: {
            cardsRawBuckets: outputs.cardsRawIndex.records.length,
            cardsNormalized: outputs.cardsNormalized.records.length,
            rulesIndex: outputs.rulesIndex.records.length,
            tokensIndex: outputs.tokensIndex.records.length
          }
        },
        null,
        2
      )}\n`
    );
  } catch (error) {
    if (error instanceof ImporterError) {
      process.stderr.write(
        `${JSON.stringify(
          {
            ok: false,
            issues: error.issues
          },
          null,
          2
        )}\n`
      );
      process.exitCode = 1;
      return;
    }

    throw error;
  }
}

type CliArgs = {
  output?: string | undefined;
  root?: string | undefined;
};

function parseArgs(args: string[]): CliArgs {
  const parsed: CliArgs = {};

  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--root") {
      const value = args[index + 1];
      if (typeof value === "string") {
        parsed.root = value;
      }
      index += 1;
      continue;
    }

    if (arg === "--output") {
      const value = args[index + 1];
      if (typeof value === "string") {
        parsed.output = value;
      }
      index += 1;
    }
  }

  return parsed;
}

function currentDirectory(): string {
  return path.dirname(fileURLToPath(import.meta.url));
}

await main();
