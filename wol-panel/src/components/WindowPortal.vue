<!-- adapted from https://github.com/Shamus03/vue-window-portal/blob/30ab2b15f9121f7bb5417cf93eb52ba5a9e4908c/src/components/VueWindowPortal.vue -->
<template>
  <div v-if="open" v-show="windowLoaded" ref="child">
    <slot />
  </div>
</template>

<script setup lang="ts">
import {
  defineModel,
  watch,
  onMounted,
  onUnmounted,
  ref,
  useTemplateRef,
  nextTick,
} from "vue";
const open = defineModel<boolean>("open", {
  required: true,
});
watch([open], (newOpen) => {
  if (newOpen) {
    openPortal();
  } else {
    closePortal();
  }
});
onMounted(() => {
  //if (open.value) {
  //openPortal();
  //}
  window.addEventListener("beforeunload", closePortal);
});
onUnmounted(() => {
  closePortal();
  window.removeEventListener("beforeunload", closePortal);
});
const windowRef = ref<Window | null>(null);
const windowLoaded = ref(false);
const childRef = useTemplateRef<HTMLDivElement>("child");
function openPortal() {
  if (windowRef.value) return;

  windowRef.value = window.open(undefined, undefined, "popup");
  windowRef.value!.document.body.appendChild(
    windowRef.value!.document.createElement("div"),
  );
  // windowRef.value!.addEventListener("unload", closePortal);
  windowRef.value!.addEventListener("beforeunload", closePortal);

  nextTick(() => {
    if (!windowRef.value || !childRef.value) return;
    windowLoaded.value = true;
    // Clear any existing content
    windowRef.value!.document.body.innerHTML = "";
    windowRef.value!.document.title = document.title;
    // Move the component into the window
    const app = document.createElement("div");
    app.id = "app";
    app.appendChild(childRef.value!);
    windowRef.value!.document.body.appendChild(app);

    // copy style
    for (const el of document.head.querySelectorAll(
      "style, link[rel=stylesheet]",
    )) {
      const clone = el.cloneNode(true);
      windowRef.value!.document.head.appendChild(clone);
    }
  });
}
function closePortal() {
  if (!windowRef.value) return;

  windowRef.value!.close();
  windowLoaded.value = false;
  open.value = false;
  // otherwise it wouldn't close for some weird reason
  setTimeout(() => (windowRef.value = null), 1);
}
</script>
