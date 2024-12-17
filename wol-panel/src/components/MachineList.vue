<script setup lang="ts">
import { CloudOffline } from "@vicons/ionicons5";
import { api_client } from "../provides";

const api = inject(api_client)!;

const {
  data,
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
</script>

<template style="padding: 10px">
  <n-list style="padding: 10px">
    <template #header> Machines </template>
    <template v-if="data">
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
