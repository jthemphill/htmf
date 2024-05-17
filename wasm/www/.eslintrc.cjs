module.exports = {
  root: true,
  env: { browser: true, es2021: true },
  extends: [
    "plugin:@typescript-eslint/strict-type-checked",
    "plugin:react-hooks/recommended",
  ],
  ignorePatterns: [
    "**/__tests__",
    ".eslintrc.cjs",
    "dist",
    "setup-vitest.ts",
    "vite.config.ts",
  ],
  parserOptions: {
    ecmaVersion: "latest",
    sourceType: "module",
    project: ["./tsconfig.json"],
    tsconfigRootDir: __dirname,
  },
  plugins: ["react-refresh"],
  rules: {},
};
