// Purpose: Keeps card-importer tests scoped to source files so build artifacts do not execute duplicate suites.

import { defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    include: ["src/**/*.test.ts"],
    exclude: ["dist/**", "node_modules/**"]
  }
});
