<script setup lang="ts">
import { Power, Stop, CloudOffline } from "@vicons/ionicons5";
import { api_client } from "../provides";
import { useThemeVars } from "naive-ui";
import type { components } from "../lib/api/v1";
import { unreachable } from "../lib/utils/rust";

const api = inject(api_client)!;

const {
  data,
  status,
  refresh: refresh_computers,
  error,
} = useAsyncData("computer-list", () =>
  api.GET("/api/machine/list").then((res) => {
    if (res.response.status !== 200) {
      throw createError({
        statusCode: res.response.status,
        statusMessage: "Backend fail",
      });
    }
    return res;
  }),
);

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

function handleSwitchMachineState(
  machine: components["schemas"]["Machine"],
  newState: components["schemas"]["State"],
) {
  switch (newState) {
    case "on":
      handle_wake(machine.name);
      break;
    case "off":
      handle_shutdown(machine.name);
      break;
    default:
      unreachable();
  }
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
              <n-switch
                size="large"
                :loading="loading"
                v-model:value="machine.state"
                checked-value="on"
                unchecked-value="off"
                @update:value="
                  (value) => handleSwitchMachineState(machine, value)
                "
              >
                <template #checked-icon>
                  <n-icon :component="Stop" />
                </template>
                <template #unchecked-icon>
                  <n-icon :component="Power" />
                </template>
              </n-switch>
              <!-- <n-button -->
              <!-- v-if="machine.state == 'off'" -->
              <!-- class="machine-action" -->
              <!-- @click="() => handle_wake(machine.name)" -->
              <!-- :loading="loading" -->
              <!-- :disabled="loading" -->
              <!-- > -->
              <!-- <template #icon> -->
              <!-- <n-icon><Power /></n-icon> -->
              <!-- </template> -->
              <!-- Wake -->
              <!-- </n-button> -->
              <!-- <n-button -->
              <!-- v-if="machine.state == 'on'" -->
              <!-- class="machine-action" -->
              <!-- @click="() => handle_shutdown(machine.name)" -->
              <!-- :loading="loading" -->
              <!-- :disabled="loading" -->
              <!-- > -->
              <!-- <template #icon> -->
              <!-- <n-icon><Stop /></n-icon> -->
              <!-- </template> -->
              <!-- Shutdown -->
              <!-- </n-button> -->
            </template>
          </n-thing>
        </n-card>
      </n-list-item>
    </template>
    <template v-else-if="status == 'idle'"> Loading... </template>
    <template v-else-if="status == 'error'">
      <n-h2>
        <n-text type="error" strong>
          <n-icon>
            <CloudOffline />
          </n-icon>
          {{ error?.name }}: {{ error?.message }}
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
