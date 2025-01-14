// import this after install `@mdi/font` package
import "@mdi/font/css/materialdesignicons.css";

import { vuetify } from "sanzu-vue";
import "vuetify/styles";

export default defineNuxtPlugin((app) => {
  app.vueApp.use(vuetify);
});
