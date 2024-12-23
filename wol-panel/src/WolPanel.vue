<script setup lang="ts">
import "./main.css";

import createClient from "openapi-fetch";
import { api_client, baseUrl, terminal_pane_provide } from "./provides";
import { DefaultTerminalState } from "./lib/utils/terminal_state";

provide(api_client, createClient({ baseUrl: baseUrl.origin }));
provide(terminal_pane_provide, DefaultTerminalState)

</script>
<template>
  <n-split direction="vertical" :size="1">
    <template #1>
      <div :style="{ height: '100%' }">
        <MachineList />
      </div>
    </template>
    <template #2>
      <div :style="{ height: '100%' }">
        <Terminal/>
      </div>
    </template>
  </n-split>
</template>
