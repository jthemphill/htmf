import { searchForWorkspaceRoot, type PluginOption } from "vite";
import { defineConfig } from "vitest/config";
import react from "@vitejs/plugin-react-swc";

// Some JS features are disabled unless you use these HTTP headers to promise not to load third-party scripts.
// This plugin only affects local development. In production, you need to configure your webserver with these HTTP headers.
// https://web.dev/coop-coep/
function setUpCrossOriginIsolation(): PluginOption {
  return {
    name: "crossOriginIsolation",
    configureServer(server): void {
      server.middlewares.use((_req, res, next) => {
        res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
        res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
        next();
      });
    },
  };
}

// https://vitejs.dev/config/
export default defineConfig({
  base: "./",
  build: {
    target: "esnext",
    sourcemap: true,
  },
  plugins: [react(), setUpCrossOriginIsolation()],
  resolve: { dedupe: ["react", "react-dom"] },
  server: {
    fs: { allow: [searchForWorkspaceRoot(process.cwd()), "../pkg"] },
  },
  test: {
    environment: "jsdom",
    setupFiles: "setup-vitest.ts",
  },
  worker: {
    format: "es",
  },
});
