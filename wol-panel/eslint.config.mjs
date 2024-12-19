// @ts-check
import withNuxt from "./.nuxt/eslint.config.mjs";

export default withNuxt(
  {
    rules: {
      "@typescript-eslint/switch-exhaustiveness-check": "warn",
    },
    files: ["*.tx", "*.vue"],
    ignores: ["*.d.ts"],
    languageOptions: {
      parserOptions: {
        project: ["./tsconfig.json"],
      },
    },
  },
  // Your custom configs here
  {
    rules: {
      "vue/html-self-closing": "off",
    },
  },
);
