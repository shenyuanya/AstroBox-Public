import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import styles from "./ConsoleOutput.module.css";

export default function ConsoleOutput() {
  const [logs, setLogs] = useState<string[]>([]);
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    invoke<string>("app_get_current_log")
      .then(content => setLogs(content.split(/\r?\n/)))
      .catch(() => {});

    const unlisten: Promise<UnlistenFn> = listen<string>("backend-log", event => {
      setLogs(prev => [...prev, event.payload]);
    });

    return () => {
      unlisten.then(fn => fn());
    };
  }, []);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: "smooth" });
  }, [logs]);

  return (
    <div className={styles.consoleContainer}>
      {logs.map((line, idx) => (
        <div key={idx} className={styles.monospace}>{line}</div>
      ))}
      <div ref={bottomRef}></div>
    </div>
  );
}
