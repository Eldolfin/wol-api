<script setup lang="ts">
import { darkTheme } from 'naive-ui'
import WolPanel from './src/main.vue'
</script>

<template>
  <div>
    <NuxtRouteAnnouncer />
    <n-config-provider :theme="darkTheme">
      <WolPanel style="height: 100vh; width: 100vw;" />
    </n-config-provider>
  </div>
</template>
