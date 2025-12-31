import { playwright } from "@vitest/browser-playwright";
import { configDefaults, defineConfig } from "vitest/config";

export default defineConfig({
  test: {
    browser: {
      enabled: true,
      provider: playwright(),
      instances: [
        { browser: "chromium" },
        // { browser: "firefox" },
        // { browser: "webkit" },
      ],
    },
    exclude: [...configDefaults.exclude, "dist"],
  },
  optimizeDeps: {
    include: ["react", "react-dom/client"],
  },
});
