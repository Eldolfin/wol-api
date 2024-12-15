<script setup lang="ts">
  import {Power, Stop} from '@vicons/ionicons5';
  import { api_client  } from '../provides';

  const api = inject(api_client)!;
  const data = await api.GET('/api/machine/list');
  const machines = data.data!.machines;

  function handle_wake(machine_name: string) {
    fetch (`${api}/${machine_name}/wake`, {method: "POST"})
  }
  function handle_shutdown(machine_name: string) {
    fetch (`${api}/${machine_name}/shutdown`, {method: "POST"})
  }
  function capitalize(s: string): string {
    return String(s[0]).toUpperCase() + String(s).slice(1);
  }
</script>

<template>
  <n-list style="padding: 10px;">
    <template #header>
      Machines
    </template>
    <n-list-item v-for="machine in machines" style="width: 90%;">
      <n-card >
        <n-thing :title="capitalize(machine.name)" :description="`ip: ${machine.config.ip} mac: ${machine.config.mac}`">
          <n-button class="machine-action" :onclick="() => handle_wake(machine.name)">
            <template #icon>
              <n-icon><Power /></n-icon>
            </template>
            Wake
          </n-button>
          <n-button class="machine-action" :onclick="() => handle_shutdown(machine.name)">
            <template #icon>
              <n-icon><Stop /></n-icon>
            </template>
            Shutdown
          </n-button>
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
