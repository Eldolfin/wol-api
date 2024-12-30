<script setup lang="ts">
const value = defineModel<string>("value", {
  required: true,
});

const { copy } = useClipboard({ source: value });
const copied = ref(false);

function handleClick() {
  copy();
  copied.value = true;
  setTimeout(() => (copied.value = false), 2000);
}
</script>
<template>
  <n-button @click="handleClick">
    {{ copied ? "Copied!" : value }}
  </n-button>
</template>
