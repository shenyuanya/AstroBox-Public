export interface DeviceMapItem {
    name: string;
    codename: string;
    chip: string;
    fetch: boolean;
}

export type DeviceMap = Record<string, DeviceMapItem>;
