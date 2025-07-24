import {PluginManifest, ProgressData, ProviderState} from "@/plugin/types";
import {Channel, invoke} from "@tauri-apps/api/core";

export default class BackendProvider {
    constructor(
        public readonly name: string,
        private onRefreshComplete: () => void
    ) {}

    async refresh(): Promise<void> {
        try {
            await invoke("plugstore_refresh", { name: this.name });
            this.onRefreshComplete();
        } catch (error) {
            console.error(`Failed to refresh provider ${this.name}:`, error);
            this.onRefreshComplete();
            throw error;
        }
    }

    async getState(): Promise<ProviderState | undefined> {
        return await invoke("plugstore_get_state", { name: this.name });
    }

    async getPage(page: number, limit: number, filter?: string): Promise<PluginManifest[]> {
        return await invoke("plugstore_get_page", {
            name: this.name,
            page,
            limit,
            filter,
        });
    }

    async getItem(plugin: string): Promise<PluginManifest> {
        return await invoke("plugstore_get_item", {
            name: this.name,
            plugin,
        });
    }

    async download(plugin: string, device: string, progressCb: Channel<ProgressData>): Promise<string> {
        return await invoke("plugstore_download", {
            name: this.name,
            plugin,
            progressCb,
        });
    }

    async getTotalItems(filter?: string): Promise<number> {
        return await invoke("plugstore_get_total_items", {
            name: this.name,
            filter,
        });
    }
}
