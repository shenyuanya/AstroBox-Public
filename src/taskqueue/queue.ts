import { useSyncExternalStore } from "react";
import { TaskItem, TaskList } from "./tasklist";

export const downloadList = new TaskList("download");
export const installList = new TaskList("install");

export function addInstallTask(task: TaskItem) {
  installList.add(task);
}

export function addDownloadTask(task: TaskItem) {
  downloadList.add(task);
}

export function useDownloadQueue() {
  // 使用 bind 确保 this 绑定正确
  const items = useSyncExternalStore(
    downloadList.subscribe.bind(downloadList),
    downloadList.snapshot.bind(downloadList),
    downloadList.snapshot.bind(downloadList),
  );
  return {
    items,
    add: downloadList.add.bind(downloadList),
    remove: downloadList.remove.bind(downloadList),
    clear: downloadList.clear.bind(downloadList),
    status: downloadList.status,
    progress: downloadList.progress,
    start: downloadList.run.bind(downloadList),
    stop: downloadList.stop.bind(downloadList),
  }
}

export function useInstallQueue() {
  // 使用 bind 确保 this 绑定正确
  const items = useSyncExternalStore(
    cb=>installList.subscribe(cb),
    ()=>installList.snapshot(),
    ()=>installList.snapshot(),
  );
  return {
    items,
    add: installList.add.bind(installList),
    remove: installList.remove.bind(installList),
    clear: installList.clear.bind(installList),
    status: installList.status,
    progress: installList.progress,
    start: installList.run.bind(installList),
    stop: installList.stop.bind(installList),
  }
}