// Purpose: opt-in proposal tests; red acceptance criteria do not enter the default suite.
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [react()],
  test: {
    environment: 'jsdom',
    setupFiles: ['src/test/setup.ts'],
    include: ['proposals/autopass-background/*.spec.jsx'],
  },
});
