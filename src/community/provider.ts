import logger from "@/log/logger";
import { ProgressData, Provider, ProviderState, SearchConfig } from "@/plugin/types";
import { Item, ResourceManifestV1 } from "@/types/ResManifestV1";
import { Channel, invoke } from "@tauri-apps/api/core";

export default class BackendProvider extends Provider {
    constructor(
        public readonly name: string,
        private onRefreshComplete: () => void
    ) {
        super();
    }

    async refresh(): Promise<void> {
        try {
            await invoke("commprov_refresh", { name: this.name });
            this.onRefreshComplete();
        } catch (error) {
            console.error(`Failed to refresh provider ${this.name}:`, error);
            this.onRefreshComplete();
            throw error;
        }
    }

    async getState(): Promise<ProviderState> {
        return await invoke("commprov_get_state", { name: this.name });
    }

    async getCategories(): Promise<string[]> {
        return await invoke("commprov_get_categories", { name: this.name });
    }

    async getPage(page: number, limit: number, search: SearchConfig): Promise<Item[]> {
        return await invoke("commprov_get_page", {
            name: this.name,
            page,
            limit,
            search,
        });
    }

    async getItem(resname: string): Promise<ResourceManifestV1> {
        logger.info(`provider ${this.name} try to getItem ${resname}`)
        return await invoke("commprov_get_item", {
            name: this.name,
            resname,
        });
    }

    async download(resname: string, device: string, progressCb: Channel<ProgressData>): Promise<string> {
        return await invoke("commprov_download", {
            name: this.name,
            resname,
            device,
            progressCb,
        });
    }

    async getTotalItems(filter?: string): Promise<number> {
        return await invoke("commprov_get_total_items", {
            name: this.name,
            filter,
        });
    }
}