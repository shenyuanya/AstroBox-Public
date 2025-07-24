import { invoke } from "@tauri-apps/api/core";

interface Logger {
  info(...args: any[]): void;
  warn(...args: any[]): void;
  error(...args: any[]): void;
}

function formatArgs(args: any[]): string {
  return args
    .map(arg => {
      if (arg instanceof Error) {
        return `${arg.name}: ${arg.message}\n${arg.stack}`;
      } else if (typeof arg === 'object') {
        try {
          return JSON.stringify(arg, null, 2);
        } catch {
          return '[Circular]';
        }
      } else {
        return String(arg);
      }
    })
    .join(' ');
}

function sendToBackend(level: string, message: string) {
  if (typeof window !== "undefined") {
    invoke('frontend_log', { level, message });
  }
}

const logger: Logger = {
  info(...args: any[]) {
    console.log(...args);
    sendToBackend('info', formatArgs(args));
  },
  warn(...args: any[]) {
    console.warn(...args);
    sendToBackend('warn', formatArgs(args));
  },
  error(...args: any[]) {
    console.error(...args);
    sendToBackend('error', formatArgs(args));
  }
};

export default logger;