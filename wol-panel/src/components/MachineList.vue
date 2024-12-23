<script setup lang="ts">
import { CloudOffline } from "@vicons/ionicons5";
import { api_client, baseUrl } from "../provides";
import { type components } from "../lib/api/v1";

type State = components["schemas"]["StoreInner"];
const machines_state = ref<State | undefined>(undefined);

const ws = new WebSocket(baseUrl.origin + "/api/machine/list_ws");
ws.onmessage = (msg) => {
  const state: State = JSON.parse(msg.data);
  machines_state.value = state;
};

const machines = computed(() => machines_state.value?.machines);
</script>

<template style="padding: 10px">
  <n-list style="padding: 10px">
    <template #header> Machines </template>
    <template v-if="machines_state !== undefined">
      <n-list-item
        v-for="i in machines!.length"
        :key="i"
        style="width: calc(100vw - 10px * 2)"
      >
        <MachineCard v-model:machine="machines![i - 1]" />
      </n-list-item>
    </template>
    <!-- <template v-else-if="status == 'idle'"> Loading... </template> -->
    <template v-else>
      <n-h2>
        <n-text type="error" strong>
          <n-icon>
            <CloudOffline />
          </n-icon>
          <!-- {{ error?.name }}: {{ error?.message }} -->
        </n-text>
      </n-h2>
    </template>
  </n-list>
</template>

<style>
.machine-action {
  margin: 10px;
}
</style>
