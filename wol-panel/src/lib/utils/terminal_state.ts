export interface TerminalState {
  currentConnectedMachineName: Ref<string | null>,
  connecting: () => boolean,
}

export const DefaultTerminalState: TerminalState = {
  connecting: () => false,
  currentConnectedMachineName: ref(null),
};
