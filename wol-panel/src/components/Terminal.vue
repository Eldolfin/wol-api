<script setup lang="ts">
import { Terminal } from "@xterm/xterm";
import { FitAddon } from "@xterm/addon-fit";
import "@xterm/xterm/css/xterm.css";
import { baseUrl } from "../provides";
import { AttachAddon } from "../lib/xterm-js-addons/attach";
import type { components } from "../lib/api/v1.d.ts";

const machineName = defineModel<string>("machineName", { required: true });
const emit = defineEmits<{
  (e: "close"): void;
}>();

const terminalParentElt = useTemplateRef<HTMLDivElement>("terminal-parent");
const terminalElt = useTemplateRef<HTMLDivElement>("terminal");
const term = new Terminal();

const { width, height } = useElementSize(terminalParentElt);

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

watchEffect(() => {
  if (machineName.value !== null) {
    term.write(`Connecting to ${machineName.value}...\n\r`);
    const ws = new WebSocket(
      baseUrl.origin + `/api/machine/ssh/${machineName.value}/connect`,
    );
    const attachAddon = new AttachAddon(ws, {
      messageWrapper: (message) => {
        const msg: components["schemas"]["SshClientMessage"] = {
          message: {
            input: message,
          },
        };
        return JSON.stringify(msg);
      },
    });
    term.loadAddon(attachAddon);
    ws.onclose = () => emit("close");
  }
});

onMounted(() => {
  term.open(terminalElt.value!);
  watchDebounced([width, height], fit, {
    immediate: true,
    debounce: 50,
  });
});

function fit() {
  terminalElt.value!.style.height = "0";
  fitAddon.fit();
}

function handleKeyDown(domEvent: KeyboardEvent) {
  if (domEvent.getModifierState("Control") && domEvent.key == "+") {
    domEvent.preventDefault();
    term.options.fontSize = (term.options.fontSize || 0) + 2;
    fit();
  } else if (domEvent.getModifierState("Control") && domEvent.key == "-") {
    domEvent.preventDefault();
    term.options.fontSize = (term.options.fontSize || 0) - 2;
    fit();
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
