<script setup lang="ts">
import type { components } from "../lib/api/v1";
import { AppsSharp } from "@vicons/ionicons5";
import { baseUrl } from "../provides";
import { ImageOutline } from "@vicons/ionicons5";

type Applications = components["schemas"]["GroupedApplication"];
type Application = components["schemas"]["ApplicationDisplay"];

const props = defineProps<{
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
        console.log(kv[1]);
        return { category: kv[0], apps: kv[1], key: kv[0], value: i };
      },
    ),
  );
});
</script>
<template>
  <n-popselect :options="[]" trigger="click">
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
        <keepalive>
          <n-virtual-list :item-size="42" :items="applications" item-resizable>
            <template #default="{ item, index }">
              <n-list-item>
                <n-h2>
                  {{ item.category }}
                </n-h2>
                <n-grid x-gap="12" :cols="4">
                  <template v-for="(application, i) in item.apps" :key="i">
                    <n-gi>
                      <n-button
                        @click="
                          () => {
                            throw 'todo';
                          }
                        "
                        size="large"
                        :style="{ height: '96px', width: '96px' }"
                      >
                        <n-grid :cols="1">
                          <n-gi>
                            <n-image
                              width="32"
                              :src="baseUrl.origin + application.icon"
                              preview-disabled
                            >
                              <template #error>
                                <n-icon color="lightGrey">
                                  <ImageOutline />
                                </n-icon>
                              </template>
                            </n-image>
                          </n-gi>
                          <n-gi>
                            <div
                              :style="{ width: '32px', wordWrap: 'break-word' }"
                            >
                              {{ application.name }}
                            </div>
                          </n-gi>
                        </n-grid>
                      </n-button>
                    </n-gi>
                  </template>
                </n-grid>
              </n-list-item>
            </template>
          </n-virtual-list>
        </keepalive>
      </n-scrollbar>
    </template>
  </n-popselect>

  <!-- <n-tree block-line :data="applications" :selectable="false" /> -->
</template>
