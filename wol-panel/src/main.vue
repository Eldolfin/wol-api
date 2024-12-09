<script setup lang="ts">
  interface Machine {
    ip: string,
    mac: string,
  }
  const endpoint = import.meta.env.VITE_WOL_API_URL;
  const api = `${endpoint}/machine`;
  const data = await (await fetch(`${api}/list`)).json();
  const machines: Map<string, Machine> = new Map(Object.entries(data.machines));
  console.log(machines)

  function handle_wake(machine_name: string) {
    fetch (`${api}/${machine_name}/wake`, {method: "POST"})
  }
  function handle_shutdown(machine_name: string) {
    fetch (`${api}/${machine_name}/shutdown`, {method: "POST"})
  }
</script>

<template>
  <n-list>
    <template #header>
      Machines
    </template>
    <n-list-item v-for="[name, machine] in machines.entries()">
      <template #prefix>
        <n-button :onclick="() => handle_wake(name)">Wake</n-button>
        <n-button :onclick="() => handle_shutdown(name)">Shutdown</n-button>
      </template>
      <n-thing :title="name" :description="`ip: ${machine.ip} mac: ${machine.mac}`">
      </n-thing>
    </n-list-item>
  </n-list>
</template>
