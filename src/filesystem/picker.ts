import { DialogFilter, open } from "@tauri-apps/plugin-dialog";

export async function pickFile(multiple: boolean, filters?: DialogFilter[]): Promise<string | string[]> {
    const file = await open({
        multiple,
        directory: false,
        ...(filters && filters.length ? { filters } : {})
    });

    return file ?? "";
}