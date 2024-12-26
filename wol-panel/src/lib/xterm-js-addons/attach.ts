/**
 * https://github.com/xtermjs/xterm.js/blob/ce095b35936844055924c9b66ec3e89aef5cbca4/addons/addon-attach/src/AttachAddon.ts
 * but modified to my needs
 */

import type { Terminal, IDisposable, ITerminalAddon } from "@xterm/xterm";

interface IAttachOptions {
  bidirectional?: boolean;
  /** Called when string data is received on the WebSocket, returns the terminal data or null if the data is not supposed to be shown on the terminal */
  dataExtractor?: (received_data: string) => string | null;
  /** Called when terminal data is about to be sent on the Websocket, it can wrap the message */
  messageWrapper?: (data: string) => string;
}

export class AttachAddon implements ITerminalAddon {
  private _socket: WebSocket;
  private _bidirectional: boolean;
  private _disposables: IDisposable[] = [];
  private _dataExtractor: (received_data: string) => string | null = (data) =>
    data;
  private _messageWrapper: (data: string) => string = (data) => data;

  constructor(socket: WebSocket, options?: IAttachOptions) {
    this._socket = socket;
    // always set binary type to arraybuffer, we do not handle blobs
    this._socket.binaryType = "arraybuffer";
    this._bidirectional = !(options && options.bidirectional === false);
    if (options?.dataExtractor) {
      this._dataExtractor = options.dataExtractor;
    }
    if (options?.messageWrapper) {
      this._messageWrapper = options.messageWrapper;
    }
  }

  public activate(terminal: Terminal): void {
    this._disposables.push(
      addSocketListener(this._socket, "message", (ev) => {
        const data: ArrayBuffer | string = ev.data;
        if (typeof data === "string") {
          const extracted = this._dataExtractor(data);
          if (extracted !== null) {
            terminal.write(data);
          }
        } else {
          terminal.write(new Uint8Array(data));
        }
      }),
    );

    if (this._bidirectional) {
      this._disposables.push(terminal.onData((data) => this._sendData(data)));
      this._disposables.push(
        terminal.onBinary((data) => this._sendBinary(data)),
      );
    }

    this._disposables.push(
      addSocketListener(this._socket, "close", () => this.dispose()),
    );
    this._disposables.push(
      addSocketListener(this._socket, "error", () => this.dispose()),
    );
  }

  public dispose(): void {
    for (const d of this._disposables) {
      d.dispose();
    }
  }

  private _sendData(data: string): void {
    if (!this._checkOpenSocket()) {
      return;
    }
    this._socket.send(this._messageWrapper(data));
  }

  private _sendBinary(data: string): void {
    if (!this._checkOpenSocket()) {
      return;
    }
    const buffer = new Uint8Array(data.length);
    for (let i = 0; i < data.length; ++i) {
      buffer[i] = data.charCodeAt(i) & 255;
    }
    this._socket.send(buffer);
  }

  private _checkOpenSocket(): boolean {
    switch (this._socket.readyState) {
      case WebSocket.OPEN:
        return true;
      case WebSocket.CONNECTING:
        throw new Error("Attach addon was loaded before socket was open");
      case WebSocket.CLOSING:
        console.warn("Attach addon socket is closing");
        return false;
      case WebSocket.CLOSED:
        throw new Error("Attach addon socket is closed");
      default:
        throw new Error("Unexpected socket state");
    }
  }
}

function addSocketListener<K extends keyof WebSocketEventMap>(
  socket: WebSocket,
  type: K,
  handler: (this: WebSocket, ev: WebSocketEventMap[K]) => any,
): IDisposable {
  socket.addEventListener(type, handler);
  return {
    dispose: () => {
      if (!handler) {
        // Already disposed
        return;
      }
      socket.removeEventListener(type, handler);
    },
  };
}
