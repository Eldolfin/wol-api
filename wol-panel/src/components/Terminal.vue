<script setup lang="ts">
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { AttachAddon } from "@xterm/addon-attach";
import { baseUrl } from "../provides";

const machineName = defineModel<string>("machineName", { required: true });
const terminalParentElt = useTemplateRef<HTMLDivElement>("terminal-parent");
const terminalElt = useTemplateRef<HTMLDivElement>("terminal");
const term = new Terminal();

const { width, height } = useElementSize(terminalParentElt);

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

watchEffect(() => {
  if (machineName.value !== null) {
    term.write(`Connecting to ${machineName.value}...\n\r`);
    const attachAddon = new AttachAddon(
      new WebSocket(
        baseUrl.origin + `/api/machine/ssh/${machineName.value}/connect`,
      ),
    );
    term.loadAddon(attachAddon);
  }
});

onMounted(() => {
  term.open(terminalElt.value!);
  watchDebounced(
    [width, height],
    () => {
      terminalElt.value!.style.height = "0";
      fitAddon.fit();
      term.refresh(0, term.rows - 1);
    },
    { immediate: true, debounce: 50 },
  );
});

function handleKeyDown(domEvent: KeyboardEvent) {
  if (domEvent.getModifierState("Control") && domEvent.key == "+") {
    domEvent.preventDefault();
    term.options.fontSize = (term.options.fontSize || 0) + 10;
  } else if (domEvent.getModifierState("Control") && domEvent.key == "-") {
    domEvent.preventDefault();
    term.options.fontSize = (term.options.fontSize || 0) - 10;
  }
}
</script>
<template>
  <div
    ref="terminal-parent"
    :style="{ display: 'flex', flexFlow: 'column', height: '100%' }"
    @keydown="handleKeyDown"
  >
    <div ref="terminal" :style="{ flex: '1 1 auto' }"></div>
  </div>
</template>
