<script setup lang="ts">
import type { components } from "../lib/api/v1";
import { AppsSharp } from "@vicons/ionicons5";

type Applications = components["schemas"]["GroupedApplication"];
type Application = components["schemas"]["ApplicationDisplay"];

const applications_raw = defineModel<Applications>("applications", {
  required: true,
});

// sorts the groups and make it easier to iterate on
const applications = computed<Map<string, Application[]>>(() => {
  const map = new Map();
  for (const [group, applications] of Object.entries(
    applications_raw.value.groups,
  )) {
    map.set(group, applications);
  }
  return new Map([...map.entries()].sort());
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
      <n-scrollbar style="max-height: 40vh" v-if="applications_raw">
        <n-grid y-gap="12" :cols="1">
          <template
            v-for="([category, applications], i) in applications.entries()"
            :key="i"
          >
            <n-gi>
              <n-h2>
                {{ category }}
              </n-h2>
              <n-grid x-gap="12" :cols="4">
                <template v-for="(application, i) in applications" :key="i">
                  <n-gi>
                    <n-button
                      @click="
                        () => {
                          throw 'todo';
                        }
                      "
                    >
                      {{ application.name }}
                      <!-- <n-image -->
                      <!-- width="30" -->
                      <!-- :src="baseUrl.origin + task.icon_url" -->
                      <!-- preview-disabled -->
                      <!-- > -->
                      <!-- <template #error> -->
                      <!-- <n-icon :size="30" color="lightGrey"> -->
                      <!-- <ImageOutlineIcon /> -->
                      <!-- </n-icon> -->
                      <!-- </template> -->
                      <!-- </n-image> -->
                    </n-button>
                  </n-gi>
                </template>
              </n-grid>
            </n-gi>
          </template>
        </n-grid>
      </n-scrollbar>
    </template>
  </n-popselect>

  <!-- <n-tree block-line :data="applications" :selectable="false" /> -->
</template>
