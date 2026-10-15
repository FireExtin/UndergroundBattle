import { cleanup } from "@testing-library/react";
import "@testing-library/jest-dom/vitest";
import { afterEach } from "vitest";

// Purpose: Registers DOM-specific Vitest matchers for the React debugger tests.
afterEach(() => {
  cleanup();
  // Each test owns one tab lifetime; remount/reload recovery stays within its test.
  sessionStorage.clear();
});
