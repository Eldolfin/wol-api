<script setup lang="ts">
  import {Power, Stop} from '@vicons/ionicons5';

  interface Machine {
    ip: string,
    mac: string,
  }
  const endpoint = import.meta.env.VITE_WOL_API_URL;
  const api = `${endpoint}/machine`;
  const data = await (await fetch(`${api}/list`)).json();
  const machines: Map<string, Machine> = new Map(Object.entries(data.machines));

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
    <n-list-item v-for="[name, machine] in machines.entries()" style="width: 90%;">
      <n-card >
        <n-thing :title="capitalize(name)" :description="`ip: ${machine.ip} mac: ${machine.mac}`">
          <n-button class="machine-action" :onclick="() => handle_wake(name)">
            <template #icon>
              <n-icon><Power /></n-icon>
            </template>
            Wake
          </n-button>
          <n-button class="machine-action" :onclick="() => handle_shutdown(name)">
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
