import AutoImport from "unplugin-auto-import/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";
import Components from "unplugin-vue-components/vite";
import vuetify, { transformAssetUrls } from "vite-plugin-vuetify";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  app: {
    head: {
      link: [{ rel: "icon", type: "image/svg+xml", href: "/favicon.svg" }],
    },
  },
  modules: [
    "nuxtjs-naive-ui",
    "@vueuse/nuxt",
    "@nuxt/eslint",
    (_options, nuxt) => {
      nuxt.hooks.hook("vite:extendConfig", (config) => {
        // @ts-expect-error
        config.plugins.push(vuetify({ autoImport: true }));
      });
    },
  ],
  ssr: false,
  devtools: { enabled: true },
  compatibilityDate: "2024-11-01",
  nitro: {
    esbuild: {
      options: {
        target: "esnext",
      },
    },
  },
  vite: {
    build: {
      target: "esnext",
      rollupOptions: {
        external: ["@sanzu/sanzu-vue"],
      },
    },
    optimizeDeps: {
      esbuildOptions: {
        target: "esnext",
      },
    },
    plugins: [
      AutoImport({
        imports: [
          {
            "naive-ui": [
              "useDialog",
              "useMessage",
              "useNotification",
              "useLoadingBar",
            ],
          },
        ],
      }),
      Components({
        resolvers: [NaiveUiResolver()],
      }),
    ],
    vue: {
      template: {
        transformAssetUrls,
      },
    },
  },
  eslint: {
    checker: true,
    config: {
      // stylistic: true,
    },
  },
});
