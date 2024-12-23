<script setup lang="ts">
const value = defineModel<string>("value", {
  required: true,
});

const { copy, copied, isSupported } = useClipboard({ source: value })

function handleClick() {
  copy()
}

</script>
<template>
  <n-button
     @click="handleClick"
     :disabled="!isSupported"
  >
    {{ copied ? 'Copied!' : value }}
  </n-button>
</template>
