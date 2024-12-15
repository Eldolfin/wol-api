<script setup lang="ts">
import { Power, Stop } from "@vicons/ionicons5";
import { api_client } from "../provides";
import { useThemeVars } from "naive-ui";
import type { components } from "../lib/api/v1";

const api = inject(api_client)!;
const data = await api.GET("/api/machine/list");
const machines = data.data!.machines;

const theme = useThemeVars();

function handle_wake(machine_name: string) {
  api.POST("/api/machine/{name}/wake", {
    params: {
      path: {
        name: machine_name,
      },
    },
  });
}

function handle_shutdown(machine_name: string) {
  fetch(`${api}/${machine_name}/shutdown`, { method: "POST" });
  api.POST("/api/machine/{name}/shutdown", {
    params: {
      path: {
        name: machine_name,
      },
    },
  });
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
              :onclick="() => handle_wake(machine.name)"
            >
              <template #icon>
                <n-icon><Power /></n-icon>
              </template>
              Wake
            </n-button>
            <n-button
              v-if="machine.state == 'on'"
              class="machine-action"
              :onclick="() => handle_shutdown(machine.name)"
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
  </n-list>
</template>

<style>
.machine-action {
  margin: 10px;
}
</style>
