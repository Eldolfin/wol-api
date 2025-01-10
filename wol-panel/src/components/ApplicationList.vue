<script setup lang="ts">
import type { components } from "../lib/api/v1";
import { AppsSharp } from "@vicons/ionicons5";

type Applications = components["schemas"]["GroupedApplication"];

const props = defineProps<{
  machine_name: string;
  applications: Applications | null | undefined;
}>();

// sorts the groups and make it easier to iterate on
const applications = computed(() => {
  const map = new Map();
  if (!props.applications) {
    return [];
  }
  for (const [group, applications] of Object.entries(
    props.applications!.groups,
  )) {
    map.set(group, applications);
  }
  return new Array(
    ...new Array(...new Map([...map.entries()].sort()).entries()).map(
      (kv, i) => {
        return { category: kv[0], apps: kv[1], key: kv[0], value: i };
      },
    ),
  );
});
</script>
<template>
  <n-popselect :options="[]" trigger="click" :style="{ padding: '12px' }">
    <n-button>
      <template #icon>
        <n-icon>
          <AppsSharp />
        </n-icon>
      </template>
      Applications
    </n-button>
    <template #empty>
      <n-scrollbar style="max-height: 60vh" v-if="props.applications">
        <KeepAlive>
          <n-virtual-list :item-size="42" :items="applications" item-resizable>
            <template #default="{ item, index }">
              <n-list-item :style="{ marginTop: '20px' }">
                <n-divider :n-if="index !== 0" />
                <n-h2>
                  {{ item.category }}
                </n-h2>
                <n-grid
                  x-gap="12"
                  cols="1 s:2 m:4 l:6 xl:7 2xl:7"
                  responsive="screen"
                >
                  <template v-for="(application, i) in item.apps" :key="i">
                    <n-gi>
                      <ApplicationButton
                        :application="application"
                        :machine_name="machine_name"
                      />
                    </n-gi>
                  </template>
                </n-grid>
              </n-list-item>
            </template>
          </n-virtual-list>
        </KeepAlive>
      </n-scrollbar>
    </template>
  </n-popselect>

  <!-- <n-tree block-line :data="applications" :selectable="false" /> -->
</template>
<style lang="postcss" scoped>
.applications-button {
  height: 120px;
  width: 96px;
  margin: 4px;
}
</style>
