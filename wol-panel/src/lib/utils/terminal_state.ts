export interface TerminalState {
  currentConnectedMachineName?: string;
  connecting: () => boolean,
}

export const DefaultTerminalState: TerminalState = {
  connecting: () => false,
};
