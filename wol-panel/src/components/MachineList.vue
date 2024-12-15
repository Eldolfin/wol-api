<script setup lang="ts">
import { Power, Stop } from "@vicons/ionicons5";
import { api_client } from "../provides";
import { useThemeVars } from "naive-ui";
import type { components } from "../lib/api/v1";

const api = inject(api_client)!;

const {
  data,
  status,
  refresh: refresh_computers,
} = useAsyncData("computer-list", () => api.GET("/api/machine/list"));

const machines = computed(() => data.value?.data?.machines);

useIntervalFn(async () => await refresh_computers(), 1000);

const theme = useThemeVars();
const loading = ref(false);

function handle_wake(machine_name: string) {
  loading.value = true;
  api
    .POST("/api/machine/{name}/wake", {
      params: {
        path: {
          name: machine_name,
        },
      },
    })
    .finally(() => (loading.value = false));
}

function handle_shutdown(machine_name: string) {
  loading.value = true;
  api
    .POST("/api/machine/{name}/shutdown", {
      params: {
        path: {
          name: machine_name,
        },
      },
    })
    .finally(() => (loading.value = false));
}

function capitalize(s: string): string {
  return String(s[0]).toUpperCase() + String(s).slice(1);
}

function machine_color(state: components["schemas"]["State"]) {
  switch (state) {
    case "on":
      return theme.value.successColorSuppl;
    case "off":
      return theme.value.errorColor;
    case "unknown":
      return theme.value.warningColorSuppl;
  }
}
</script>

<template style="padding: 10px">
  <n-list style="padding: 10px">
    <template #header> Machines </template>
    <template v-if="status == 'success' || status == 'pending'">
      <n-list-item
        v-for="machine in machines"
        style="width: calc(100vw - 10px * 2)"
      >
        <n-card :style="{ borderRadius: theme.borderRadius }">
          <n-thing>
            <template #avatar>
              <n-avatar
                :style="{ backgroundColor: machine_color(machine.state) }"
              >
                <n-icon>
                  <template v-if="machine.state == 'unknown'">?</template>
                  <Power v-else />
                </n-icon>
              </n-avatar>
            </template>
            <template #header> {{ capitalize(machine.name) }} </template>
            <template #description>
              {{ `state: ${machine.state}` }}
              <br />
              {{ `ip: ${machine.config.ip}` }}
              <br />
              {{ `mac: ${machine.config.mac}` }}
            </template>
            <template #action>
              <n-button
                v-if="machine.state == 'off'"
                class="machine-action"
                @click="() => handle_wake(machine.name)"
                :loading="loading"
                :disabled="loading"
              >
                <template #icon>
                  <n-icon><Power /></n-icon>
                </template>
                Wake
              </n-button>
              <n-button
                v-if="machine.state == 'on'"
                class="machine-action"
                @click="() => handle_shutdown(machine.name)"
                :loading="loading"
                :disabled="loading"
              >
                <template #icon>
                  <n-icon><Stop /></n-icon>
                </template>
                Shutdown
              </n-button>
            </template>
          </n-thing>
        </n-card>
      </n-list-item>
    </template>
    <template v-else-if="status == 'idle'"> Loading... </template>
    <template v-else-if="status == 'error'"> TODO: error? </template>
  </n-list>
</template>

<style>
.machine-action {
  margin: 10px;
}
</style>
