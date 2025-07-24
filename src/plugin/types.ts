import { Item, ResourceManifestV1 } from "@/types/ResManifestV1";
import { Channel } from "@tauri-apps/api/core";
export enum ProviderState {
    Ready = "Ready",
    Updating = "Updating",
    Failed = "Failed",
}

export abstract class Provider {
    abstract readonly name: string;
    abstract refresh(): Promise<void>;
    abstract getState(): Promise<ProviderState>;
    abstract getPage(page: number, limit: number, search:SearchConfig): Promise<Item[]>;
    abstract getItem(name: string): Promise<ResourceManifestV1>;
    abstract download(name: string, device: string, progressCb: Channel<ProgressData>): Promise<string>;
    abstract getTotalItems(filter?: string): Promise<number>;
}

export interface SearchConfig {
    filter?: string;
    sort?: string;
    device?: string;
}

export interface ProgressData {
    progress: number,
    status: string,
    status_text: string
}

export interface PluginManifest {
    disabled?: boolean;
    name?: string;
    version?: string;
    description?: string;
    author?: string;
    website?: string;
    icon?: string;
    api_level?: number;
    permissions?: string[];
}

export interface PluginState{
    disabled: boolean,
    icon_b64: string,
}
export interface PluginUIButton {
    primary: boolean;
    text: string;
    callback_fun_id: string;
}

export interface PluginUIDropdown {
    options: string[];
    callback_fun_id: string;
}

export interface PluginUIInput {
    text: string;
    callback_fun_id: string;
}

export type PluginUINodeContent =
    | { type: "Text"; value: string }
    | { type: "Button"; value: PluginUIButton }
    | { type: "Dropdown"; value: PluginUIDropdown }
    | { type: "Input"; value: PluginUIInput }
    | { type: "HtmlDocument"; value: string };

export interface PluginUINode {
    node_id: string;
    visibility: boolean;
    disabled: boolean;
    content: PluginUINodeContent;
}
