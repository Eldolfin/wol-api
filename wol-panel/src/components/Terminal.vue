<script setup lang="ts">
import { Terminal } from '@xterm/xterm';
import {FitAddon} from "@xterm/addon-fit";
import '@xterm/xterm/css/xterm.css';
import { AttachAddon } from '@xterm/addon-attach';
import { baseUrl, terminal_pane_provide } from '../provides';

const terminalState = inject(terminal_pane_provide)!;

const terminalParentElt = useTemplateRef<HTMLDivElement>("terminal-parent");
const terminalElt = useTemplateRef<HTMLDivElement>("terminal");
const term = new Terminal();

const { width, height } = useElementSize(terminalParentElt);

const fitAddon = new FitAddon();
term.loadAddon(fitAddon);

watchEffect(() => {
  if (terminalState.currentConnectedMachineName.value !== null) {
    term.write(`Connecting to ${terminalState.currentConnectedMachineName.value}...\n`)
    const attachAddon = new AttachAddon(
    new WebSocket(baseUrl + `/api/machine/ssh/${terminalState.currentConnectedMachineName.value}/connect`)
    );
    term.loadAddon(attachAddon);
  }
})

onMounted(()=> {
  term.open(terminalElt.value!);
  term.write('Hello from \x1B[1;3;31mxterm.js\x1B[0m $ ')

  watch([width, height], () => {
    fitAddon.fit();
  }, {immediate: true})
})

</script>
<template>
  <div ref="terminal-parent" :style="{height: '100%'}">
    <div>
    Currently connected to '{{terminalState.currentConnectedMachineName}}'
    </div>
    <div ref="terminal"></div>
  </div>
</template>
