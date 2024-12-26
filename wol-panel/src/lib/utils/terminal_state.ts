export interface TerminalState {
  connectedMachines: Ref<TerminalSession[]>;
  connectToMachine: (machineName: string) => void;
}

export interface TerminalSession {
  machineName: string;
  sessionId: number;
}
