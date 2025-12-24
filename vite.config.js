import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";
// Some JS features are disabled unless you use these HTTP headers to promise not to load third-party scripts.
// This plugin only affects local development. In production, you need to configure your webserver with these HTTP headers.
// https://web.dev/coop-coep/
function setUpCrossOriginIsolation() {
    return {
        name: "crossOriginIsolation",
        configureServer(server) {
            server.middlewares.use((_req, res, next) => {
                res.setHeader("Cross-Origin-Opener-Policy", "same-origin");
                res.setHeader("Cross-Origin-Embedder-Policy", "require-corp");
                next();
            });
        },
        configurePreviewServer(server) {
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
    plugins: [
        react({
            babel: {
                plugins: [["babel-plugin-react-compiler", { target: "19" }]],
            },
        }),
        setUpCrossOriginIsolation(),
    ],
    resolve: { dedupe: ["react", "react-dom"] },
    worker: {
        format: "es",
    },
});
