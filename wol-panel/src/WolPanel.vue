<script setup lang="ts">
import "./main.css";

import createClient from "openapi-fetch";
import { api_client, baseUrl, terminal_pane_provide } from "./provides";
import type { TerminalSession } from "./lib/utils/terminal_state";

const connectedMachines = ref<TerminalSession[]>([]);
const focusedTerminal = ref<number>(0);
const splitSize = ref(1);
let sessionIdIncr = 0;

provide(terminal_pane_provide, {
  connectedMachines,
  connectToMachine: (machineName: string) => {
    const newSession = { machineName, sessionId: sessionIdIncr++ };
    connectedMachines.value.push(newSession);
    focusedTerminal.value = newSession.sessionId;
  },
});
provide(api_client, createClient({ baseUrl: baseUrl.origin }));

function handleCloseTerminal(sessionId: number) {
  const index = connectedMachines.value.findIndex(
    (session) => session.sessionId == sessionId,
  );
  if (connectedMachines.value.length == 1) focusedTerminal.value = 0;
  else if (index == 0)
    focusedTerminal.value = connectedMachines.value[1].sessionId;
  else focusedTerminal.value = connectedMachines.value[index - 1].sessionId;
  connectedMachines.value.splice(index, 1);
}

watch(
  [connectedMachines],
  () => {
    if (connectedMachines.value.length > 0 && splitSize.value > 0.9) {
      splitSize.value = 0.5;
    }
    if (connectedMachines.value.length == 0) {
      splitSize.value = 1;
    }
  },
  { deep: true },
);
</script>
<template>
  <n-card
    :style="{ height: '100%' }"
    :bordered="false"
    content-style="padding: 0;"
  >
    <n-split
      v-model:size="splitSize"
      direction="vertical"
      :style="{ height: '100%' }"
      :disabled="connectedMachines.length === 0"
    >
      <template #1>
        <n-card :style="{ height: '100%' }">
          <MachineList :style="{ height: '100%' }" />
        </n-card>
      </template>
      <template v-if="connectedMachines.length > 0" #2>
        <n-card :style="{ height: '100%' }">
          <n-tabs
            v-model:value="focusedTerminal"
            type="card"
            closable
            :style="{ height: '100%' }"
            @close="handleCloseTerminal"
          >
            <n-tab-pane
              v-for="session in connectedMachines"
              :key="session.sessionId"
              :tab="session.machineName"
              :name="session.sessionId"
              :style="{ height: '100%' }"
              display-directive="show"
            >
              <Terminal
                :style="{ height: '100%' }"
                :machine-name="session.machineName"
                @close="() => handleCloseTerminal(session.sessionId)"
              />
            </n-tab-pane>
          </n-tabs>
        </n-card>
      </template>
    </n-split>
  </n-card>
</template>
