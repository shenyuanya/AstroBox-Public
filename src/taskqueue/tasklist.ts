import { ResourceType } from "@/device/install";
import logger from "@/log/logger";
import { invoke } from "@tauri-apps/api/core";

export interface TaskItem {
    id: string;
    name: string;
    description: string;
    type?: ResourceType
    icon: React.ComponentType<any>;
    payload?: TaskItemPayload;
    action?: (ctx: TaskActionContext) => Promise<void>;
}
export interface TaskActionContext {
    update: (partial: Partial<TaskItem>) => void;
    get: () => TaskItem | undefined;
}
export interface TaskItemPayload {
    url: string;
    progress: number;
    status: "pending" | "running" | "success" | "error";
    fileSize?: string;
    progressDesc?: string;
}
export class TaskList {
    type: "download" | "install";
    items: TaskItem[] = [];
    private idSet = new Set<string>()
    private listeners = new Set<() => void>();
    status: "pending" | "running" | "stopping" = "pending";
    get progress(): number {
        if(this.items.length === 0)return 100;
        return this.items.reduce((acc, cur) => acc + (cur.payload?.progress ?? 0), 0) / this.items.length;
    }
    constructor(type: "download" | "install") {
        this.type = type;
    }

    private notify() {
        this.listeners.forEach(l => l());
    }
    update(id: string, partial: Partial<TaskItem>) {
        this.items = this.items.map((i) => (i.id === id ? { ...i, ...partial } : i));
        this.notify()
    }
    subscribe(onStoreChange: () => void) {
        this.listeners.add(onStoreChange);
        return () => this.listeners.delete(onStoreChange);
    }
    snapshot() {
        return this.items;
    }
    add(item: TaskItem) {
        if (this.idSet.has(item.id)) return
        this.idSet.add(item.id)
        this.items = [...this.items, item]
        this.notify()
        if (this.type === "download" && this.status === "pending") this.run()
    }
    remove(id: string) {
        this.items = this.items.filter((i) => i.id !== id);
        this.idSet.delete(id)
        if (this.items.length === 0) this.status = "pending"
        this.notify()
    }
    clear() {
        this.items = [];
        this.status = "pending";
        this.idSet.clear()
        this.notify()
    }

    async run() {
        this.status = "running"
        let errored = false
        this.notify()
        for (const item of this.items) {
            if (!item.payload?.status || !["pending", "error"].includes(item.payload.status)) continue
            //@ts-ignore
            if (this.status === "stopping") {
                this.status = "pending"
                this.items=[...this.items]
                return this.notify()
            }
            try {
                await item.action?.({
                update: (partial) => this.update(item.id, partial),
                get: () => this.items.find(i => i.id === item.id),
            })
            } catch (e: any) {
                logger.error("run task failed", e)
                errored = true
                this.update(item.id, {
                    payload: {
                        status: "error",
                        progress: item.payload.progress,
                        url: item.payload.url,
                        progressDesc: e instanceof Error ? e.message : e,
                    }
                })
            }
        }
        const config = await invoke<any>("app_get_config");
        if (config.disable_auto_clean) this.items = this.items.map(i => {
            if (i.payload) i.payload.status = "pending"
            return i
        })
        if ((!config.disable_auto_clean||this.type=="download")&&!errored) this.clear()
        else this.items = [...this.items]
        this.status = "pending"
        this.notify()
    }
    stop() {
        if (this.status == "running") {
            this.status = "stopping"
            this.items = [...this.items]
            this.notify()
        }
    }
}


