<script setup lang="ts">
import { useThemeVars } from "naive-ui";
import type { components } from "../lib/api/v1";
import { unreachable } from "../lib/utils/rust";
import {
  Power,
  ImageOutline as ImageOutlineIcon,
  LogInOutline,
} from "@vicons/ionicons5";
import { api_client, terminal_pane_provide } from "../provides";

const machine = defineModel<components["schemas"]["Machine"]>("machine", {
  required: true,
});

const terminalState = inject(terminal_pane_provide)!;
const theme = useThemeVars();
const loading = ref(false);
const terminalOpened = ref(false);
const button_blocked = computed(
  () =>
    loading.value ||
    machine.value.state == "pending_off" ||
    machine.value.state == "pending_on" ||
    machine.value.state == "unknown",
);
const api = inject(api_client)!;

function handle_wake() {
  loading.value = true;
  api
    .POST("/api/machine/{name}/wake", {
      params: {
        path: {
          name: machine.value.name,
        },
      },
    })
    .finally(() => (loading.value = false));
}

function handle_shutdown() {
  loading.value = true;
  api
    .POST("/api/machine/{name}/shutdown", {
      params: {
        path: {
          name: machine.value.name,
        },
      },
    })
    .finally(() => (loading.value = false));
}

function handleSwitchMachineState(newState: boolean) {
  switch (newState) {
    case true:
      handle_wake();
      break;
    case false:
      handle_shutdown();
      break;
    default:
      unreachable();
  }
}

function handleQueueTask(taskId: number) {
  api.POST("/api/machine/{name}/task", {
    params: {
      path: {
        name: machine.value.name,
      },
    },
    body: { id: taskId },
  });
}

function capitalize(s: string): string {
  return String(s[0]).toUpperCase() + String(s).slice(1);
}
const name = computed(() => capitalize(machine.value.name));
const state = computed(() => {
  switch (machine.value?.state) {
    case "unknown":
    case "on":
    case "off":
      return capitalize(machine.value.state);
    case "pending_on":
      return "Turning on...";
    case "pending_off":
      return "Turning off...";
    default:
      return unreachable();
  }
});

const avatar_color = computed(() => {
  switch (machine.value?.state) {
    case "on":
      return theme.value.successColorSuppl;
    case "off":
      return theme.value.errorColor;
    default:
      return theme.value.warningColorSuppl;
  }
});

const parsed_ip = computed(() => {
  const [ssh_host, ssh_port] = machine.value.config.ip.split(":");
  return { ssh_host, ssh_port: ssh_port || "22" };
});

function handleOpenTerminal() {
  terminalOpened.value = true;
  terminalState.connectToMachine(machine.value.name);
}
</script>
<template>
  <n-card :style="{ borderRadius: theme.borderRadius }" hoverable>
    <n-thing>
      <template #avatar>
        <n-avatar :style="{ backgroundColor: avatar_color }">
          <n-icon>
            <template v-if="machine.state == 'unknown'">?</template>
            <Power v-else />
          </n-icon>
        </n-avatar>
      </template>
      <template #header> {{ name }} </template>
      <template #description>
        {{ `state: ${state}` }}
        <br />
        {{ `mac: ${machine.config.mac}` }}
        <br />
        <n-space vertical>
          <CopiableButton :value="parsed_ip.ssh_host" />
          <CopiableButton
            :value="`ssh -p ${parsed_ip.ssh_port} oscar@${parsed_ip.ssh_host}`"
          />
        </n-space>
      </template>
      <template #action>
        <n-button-group>
          <n-space size="small">
            <n-switch
              size="large"
              :loading="button_blocked"
              :value="machine.state === 'on'"
              @update:value="handleSwitchMachineState"
            />
            <n-button
              v-for="(task, i) in machine.config.tasks"
              :key="i"
              @click="handleQueueTask(i)"
            >
              <n-image width="30" :src="task.icon_url" preview-disabled>
                <template #error>
                  <n-icon :size="30" color="lightGrey">
                    <ImageOutlineIcon />
                  </n-icon>
                </template>
              </n-image>
            </n-button>
            <n-button :loading="loading" @click="handleOpenTerminal">
              <template #icon>
                <n-icon>
                  <LogInOutline />
                </n-icon>
              </template>
              Connect via ssh
            </n-button>
          </n-space>
        </n-button-group>
      </template>
    </n-thing>
  </n-card>
</template>
