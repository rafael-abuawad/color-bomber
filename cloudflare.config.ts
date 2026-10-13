import { defineConfig } from "cf/config";

export default defineConfig({
  worker: {
    name: "color-bomber",
    compatibilityDate: "2026-10-12",
    observability: {
      enabled: true,
      traces: {
        enabled: true,
      },
    },
  },
});
