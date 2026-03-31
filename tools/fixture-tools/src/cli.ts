// Purpose: Generates the normalized JSON artifact from the shared CardLogic DSL fixtures.

import { generateNormalizedFixtures } from "./contracts.js";

const normalized = await generateNormalizedFixtures();
process.stdout.write(
  `${JSON.stringify(
    {
      ok: true,
      schemaVersion: normalized.schemaVersion,
      recordCount: normalized.records.length
    },
    null,
    2
  )}\n`
);
