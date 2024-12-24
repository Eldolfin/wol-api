<script setup lang="ts">
import "./main.css";

import createClient from "openapi-fetch";
import { api_client, baseUrl, terminal_pane_provide } from "./provides";
import { DefaultTerminalState } from "./lib/utils/terminal_state";

provide(api_client, createClient({ baseUrl: baseUrl.origin }));
provide(terminal_pane_provide, DefaultTerminalState)

</script>
<template>
  <n-split direction="vertical" style="height: 100vh; width: 100vw">
    <template #1>
      <n-card :style="{ height: '100%' }">
        <MachineList :style="{ height: '100%' }"/>
      </n-card>
    </template>
    <template #2>
      <n-card :style="{ height: '100%' }" closable title="Remote Terminal">
        <Terminal :style="{ height: '100%' }"/>
      </n-card>
    </template>
  </n-split>
</template>
