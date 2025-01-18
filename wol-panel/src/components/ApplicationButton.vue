<script setup lang="ts">
import type { components } from "../lib/api/v1";
import { api_client, baseUrl } from "../provides";
import { ImageOutline } from "@vicons/ionicons5";
type Application = components["schemas"]["ApplicationDisplay"];

const props = defineProps<{ application: Application; machineName: string }>();
const api = inject(api_client)!;

async function handleClick() {
  // TODO: handle error?
  await api.POST("/api/machine/{name}/open_application/{application_name}", {
    params: {
      path: {
        name: props.machineName,
        application_name: props.application.name,
      },
    },
  });
}
</script>
<template>
  <n-button size="large" :class="['applications-button']" @click="handleClick">
    <n-grid :cols="1">
      <n-gi>
        <n-image
          width="64"
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
      <n-gi
        :style="{
          width: '100%',
        }"
      >
        <div
          :style="{
            overflowWrap: 'anywhere',
            wordBreak: 'break-all',
            textWrap: 'balance',
            fontSize: '0.8rem',
            width: '100%',
          }"
        >
          {{ application.name }}
        </div>
      </n-gi>
    </n-grid>
  </n-button>
</template>
