import { Provider } from "@/plugin/types";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useSyncExternalStore } from "react";
import BackendProvider from "./provider";

export class ProviderManager {
    private providers = new Map<string, Provider>();
    private listeners = new Set<() => void>();

    private snapshot: {
        providers: Provider[];
        loading: boolean;
        errors: any[];
        lastUpdated: number;
    } = {
            providers: [],
            loading: false,
            errors: [],
            lastUpdated: 0,
        };

    private notify() {
        this.listeners.forEach(l => l());
    }

    public setState(updater: (prevState: typeof this.snapshot) => typeof this.snapshot) {
        const newState = updater(this.snapshot);
        if (newState !== this.snapshot) {
            this.snapshot = newState;
            this.notify();
        }
    }

    public subscribe(callback: () => void): () => void {
        this.listeners.add(callback);
        return () => {
            this.listeners.delete(callback);
        };
    }

    private handleProviderRefresh = () => {
        console.log("A provider has completed its refresh. Triggering UI update.");
        this.setState(s => ({
            ...s,
            lastUpdated: Date.now()
        }));
    };

    public async refreshAll() {
        if (this.providers.size === 0) {
            this.setState(s => ({ ...s, loading: false }));
            return;
        }

        this.setState(s => ({ ...s, loading: true, error: null }));
        const providers = Array.from(this.providers.values());
        for (const provider of providers) {
            try {
                await provider.refresh();
            } catch (e) {
                this.setState(s => ({ ...s,  errors: [...s.errors,{e,name:provider.name}] }));
            }
        }
        this.setState(s => ({ ...s, loading: false }));
    }

    private async refreshProviderList() {
        const names: string[] = await invoke("commprov_get_providers");
        const existing = new Set(this.providers.keys());
        const incoming = new Set(names);
        let changed = false;

        names.forEach(name => {
            if (!this.providers.has(name)) {
                this.providers.set(name, new BackendProvider(name, this.handleProviderRefresh));
                changed = true;
            }
        });

        for (const key of Array.from(existing)) {
            if (!incoming.has(key)) {
                this.providers.delete(key);
                changed = true;
            }
        }

        if (changed) {
            this.setState(s => ({
                ...s,
                providers: Array.from(this.providers.values())
            }));
        }
    }

    public async listAsync() {
        if (this.snapshot.loading) return;

        this.setState(s => ({ ...s, loading: true, errors: [] }));

        try {
            await this.refreshProviderList();
            await this.refreshAll();
        } catch (e) {
            console.error("Error in listAsync process:", e);
            this.setState(s => ({ ...s, loading: false, errors: [...s.errors,{e}] }));
        }
    }

    public getSnapshot = () => this.snapshot;

    public get(name: string) { return this.providers.get(name); }

    public useProviders() {
        useEffect(() => {
            this.listAsync();
        }, []);

        return useSyncExternalStore(
            this.subscribe.bind(this),
            this.getSnapshot,
            this.getSnapshot
        );
    }
}

export const providerManager = new ProviderManager();