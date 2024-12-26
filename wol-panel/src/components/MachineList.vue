<script setup lang="ts">
import { CloudOffline } from "@vicons/ionicons5";
import { wsUrl } from "../provides";
import { type components } from "../lib/api/v1";

type State = components["schemas"]["StoreInner"];
const machines_state = ref<State | undefined>(undefined);

// TODO: replace with useWebSocket
const ws = new WebSocket(wsUrl + "/api/machine/list_ws");
ws.onmessage = (msg) => {
  const state: State = JSON.parse(msg.data);
  machines_state.value = state;
};

const machines = computed(() => machines_state.value?.machines);
</script>

<template>
  <n-list>
    <template #header> Machines </template>
    <template v-if="machines_state !== undefined">
      <n-list-item v-for="i in machines!.length" :key="i">
        <MachineCard v-model:machine="machines![i - 1]" />
      </n-list-item>
    </template>
    <template v-else>
      <n-h2>
        <n-text type="error" strong>
          <n-icon>
            <CloudOffline />
          </n-icon>
        </n-text>
      </n-h2>
    </template>
  </n-list>
</template>
