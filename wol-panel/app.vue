<script setup lang="ts">
import { darkTheme } from "naive-ui";
import WolPanel from "./src/WolPanel.vue";
</script>

<template>
  <div>
    <NuxtRouteAnnouncer />
    <n-config-provider :theme="darkTheme">
      <n-notification-provider>
        <WolPanel style="height: 100vh; width: 100vw" />
      </n-notification-provider>
    </n-config-provider>
  </div>
</template>
