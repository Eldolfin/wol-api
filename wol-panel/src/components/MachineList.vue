<script setup lang="ts">
import { wsUrl } from "../provides";
import type { components } from "../lib/api/v1";

type State = components["schemas"]["StoreInner"];

const loadingBar = useLoadingBar();
const errored = ref(false);
const { data } = useWebSocket(wsUrl + "/api/machine/list_ws", {
  autoReconnect: true,
  onError: () => {
    errored.value = true;
    loadingBar.error();
  },
  onConnected: () => loadingBar.finish(),
});

const machines = computed(() => {
  const machines_data: State = JSON.parse(data.value);
  return machines_data?.machines;
});

onMounted(() => {
  loadingBar.start();
});
</script>

<template>
  <div
    :style="{
      height: '100%',
      display: 'flex',
      alignItems: 'center',
      justifyContent: 'center',
    }"
  >
    <n-list>
      <template v-if="machines !== undefined" #header> Machines </template>
      <template v-if="machines !== undefined">
        <n-list-item v-for="i in machines!.length" :key="i">
          <MachineCard v-model:machine="machines![i - 1]" />
        </n-list-item>
      </template>
      <template v-else-if="errored">
        <n-result
          status="500"
          title="Server Disconnected"
          description="I'm trying to reconnect..."
          size="huge"
        >
        </n-result>
      </template>
      <template v-else>
        <n-list-item v-for="i in 2" :key="i">
          <n-skeleton height="243px" width="700px" />
        </n-list-item>
      </template>
    </n-list>
  </div>
</template>
<style scoped>
.wol-machine-card {
  @media only screen and (max-width: 600px) {
    width: 100%;
  }
  @media only screen and (min-width: 600px) {
    width: 100%;
  }
  @media only screen and (min-width: 768px) {
    width: 700px;
  }
  @media only screen and (min-width: 992px) {
    width: 700px;
  }
  @media only screen and (min-width: 1200px) {
    width: 700px;
  }
}
</style>
