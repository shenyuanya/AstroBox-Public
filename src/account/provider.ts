import { invoke } from "@tauri-apps/api/core";

export interface Account {
    id: string;
    username: string;
    avatar: string;
    data?: Record<string, string>;
}

export default class AccountBackend {
    constructor(public readonly name: string) {}

    async list(): Promise<Account[]> {
        return await invoke("account_get", { name: this.name });
    }

    async add(account: Account) {
        await invoke("account_add", { name: this.name, account });
    }

    async remove(index: number) {
        await invoke("account_remove", { name: this.name, index });
    }
}
