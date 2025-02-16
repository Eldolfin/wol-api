<script setup lang="ts">
import type { components } from "../lib/api/v1";
import { AppsSharp } from "@vicons/ionicons5";
import { Search } from "@vicons/ionicons5";
import type { IFuseOptions } from "fuse.js";
import Fuse from "fuse.js";

type Applications = components["schemas"]["GroupedApplication"];

const props = defineProps<{
  machineName: string;
  applications: Applications | null | undefined;
}>();

const searchTerm = ref("");

const fuseOptions = { keys: ["name"] } satisfies IFuseOptions<any>;

// sorts the groups and make it easier to iterate on
const applications = computed(() => {
  const map = new Map();
  if (!props.applications) {
    return [];
  }
  for (const [group, applications] of Object.entries(
    props.applications!.groups,
  )) {
    let filtered = applications;
    if (searchTerm.value.trim().length > 0) {
      filtered = new Fuse(applications, fuseOptions)
        .search(searchTerm.value)
        .map((res) => res.item);
    }
    if (filtered.length > 0) map.set(group, filtered);
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
      <n-scrollbar v-if="props.applications" style="max-height: 60vh">
        <n-input v-model:value="searchTerm" placeholder="Search">
          <template #prefix>
            <n-icon :component="Search" />
          </template>
        </n-input>
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
                  <template v-for="application in item.apps" :key="i">
                    <n-gi>
                      <ApplicationButton
                        :application="application"
                        :machine-name="machineName"
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
