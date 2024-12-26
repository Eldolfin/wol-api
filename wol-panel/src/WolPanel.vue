<script setup lang="ts">
import "./main.css";

import createClient from "openapi-fetch";
import { api_client, baseUrl, terminal_pane_provide } from "./provides";

const connectedMachines = ref([]);
const focusedTerminal = ref(0);
const splitSize = ref(1);

provide(terminal_pane_provide, { connectedMachines });
provide(api_client, createClient({ baseUrl: baseUrl.origin }));

function handleCloseTerminal(index: number) {
  connectedMachines.value.splice(index, 1);
}

watch(
  [connectedMachines],
  () => {
    if (connectedMachines.value.length > 0 && splitSize.value > 0.9) {
      splitSize.value = 0.5;
    }
  },
  { deep: true },
);
</script>
<template>
  <n-split
    v-model:size="splitSize"
    direction="vertical"
    :style="{ height: '100%' }"
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
            v-for="(machineName, i) in connectedMachines"
            :key="i"
            :tab="machineName"
            :name="i"
            :style="{ height: '100%' }"
            display-directive="show"
          >
            <KeepAlive>
              <Terminal
                :style="{ height: '100%' }"
                :machine-name="machineName"
              />
            </KeepAlive>
          </n-tab-pane>
        </n-tabs>
      </n-card>
    </template>
  </n-split>
</template>
