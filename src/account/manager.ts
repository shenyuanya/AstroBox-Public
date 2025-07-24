import { invoke } from "@tauri-apps/api/core";
import BackendProvider from "./provider";
import { useSyncExternalStore } from "react";

export class AccountManager {
    private providers = new Map<string, BackendProvider>();
    private listeners = new Set<() => void>();

    private notify() {
        this.listeners.forEach(l => l());
    }

    async refresh() {
        const names: string[] = await invoke("account_get_providers");
        const existing = new Set(names);

        names.forEach(name => {
            if (!this.providers.has(name)) {
                this.providers.set(name, new BackendProvider(name));
            }
        });

        for (const key of Array.from(this.providers.keys())) {
            if (!existing.has(key)) {
                this.providers.delete(key);
            }
        }

        this.notify();
    }

    list() {
        return Array.from(this.providers.values());
    }

    get(name: string) {
        return this.providers.get(name);
    }

    add(name: string, provider: BackendProvider) {
        this.providers.set(name, provider);
        this.notify();
    }

    remove(name: string) {
        this.providers.delete(name);
        this.notify();
    }

    useProviders() {
        const snapshot = () => this.list();
        return useSyncExternalStore(
            cb => {
                this.listeners.add(cb);
                return () => this.listeners.delete(cb);
            },
            snapshot,
            snapshot,
        );
    }
}

export const accountManager = new AccountManager();
