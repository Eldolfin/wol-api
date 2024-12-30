<script setup lang="ts">
import { CloudOffline } from "@vicons/ionicons5";
import { wsUrl } from "../provides";
import type { components } from "../lib/api/v1";

type State = components["schemas"]["StoreInner"];

const { data } = useWebSocket(wsUrl + "/api/machine/list_ws", {
  autoReconnect: true,
});

const machines = computed(() => {
  const machines_data: State = JSON.parse(data.value);
  return machines_data?.machines;
});
</script>

<template>
  <n-list>
    <template #header> Machines </template>
    <template v-if="machines !== undefined">
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
