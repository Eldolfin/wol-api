<script setup lang="ts">
import WolPanel from "./src/main.vue";
import { darkTheme } from 'naive-ui'


</script>

<template>
  <div>
    <NuxtRouteAnnouncer />
    <n-config-provider :theme="darkTheme">
      <WolPanel style="height: 100vh; width: 100vw;" />
    </n-config-provider>
  </div>
</template>
