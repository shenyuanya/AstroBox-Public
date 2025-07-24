import { useEffect } from "react";
import useSWR from "swr";
import { invoke } from "@tauri-apps/api/core";
import { DeviceMap } from "@/types/DeviceMap";
import logger from "@/log/logger";

const STORAGE_KEY = "device_map";

export default function useDeviceMap() {
    const fallback = typeof window !== 'undefined' ? localStorage.getItem(STORAGE_KEY) : null;
    const initial = fallback ? JSON.parse(fallback) as DeviceMap : undefined;
    const { data } = useSWR<DeviceMap>("officialprov_get_device_map", () => invoke<DeviceMap>("officialprov_get_device_map"), { fallbackData: initial });

    useEffect(() => {
        if (data) {
            logger.info(`got device map: ${JSON.stringify(data)}`);
            localStorage.setItem(STORAGE_KEY, JSON.stringify(data));
        }
    }, [data]);

    return data;
}
