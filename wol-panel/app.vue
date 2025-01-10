<script setup lang="ts">
import { darkTheme } from "naive-ui";
import WolPanel from "./src/WolPanel.vue";
const showDevtools = import.meta.env.DEV;
</script>
<template>
  <div>
    <NuxtRouteAnnouncer />
    <n-config-provider :theme="darkTheme">
      <template v-if="showDevtools">
        <n-theme-editor>
          <WolPanel />
        </n-theme-editor>
      </template>
      <template v-else>
        <WolPanel />
      </template>
    </n-config-provider>
  </div>
</template>
