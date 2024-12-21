import AutoImport from "unplugin-auto-import/vite";
import { NaiveUiResolver } from "unplugin-vue-components/resolvers";
import Components from "unplugin-vue-components/vite";

// https://nuxt.com/docs/api/configuration/nuxt-config
export default defineNuxtConfig({
  modules: ["nuxtjs-naive-ui", "@vueuse/nuxt", "@nuxt/eslint"],
  ssr: false,
  devtools: { enabled: true },
  compatibilityDate: "2024-11-01",
  vite: {
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
  },
  eslint: {
    // checker: true,
    config: {
      // stylistic: true,
    },
  },
});
