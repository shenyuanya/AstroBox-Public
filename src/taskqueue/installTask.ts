import { ResourceType } from "@/device/install";
import { TaskActionContext, TaskItem } from "@/taskqueue/tasklist";
import { SendMassCallBackData } from "@/types/bluetooth";
import { AppsRegular, BoxMultipleRegular, ClockArrowDownloadRegular } from "@fluentui/react-icons";
import { Channel, invoke } from "@tauri-apps/api/core";

// watchface任务
export function createWatchFaceInstallTask(id: string, name: string, filePath: string): TaskItem {
    return {
        id,
        name,
        description: "installTask.watchface.description",
        icon: ClockArrowDownloadRegular, // 替换为实际的图标组件
        type: ResourceType.WatchFace,
        payload: {
            url: filePath,
            progress: 0,
            status: "pending",
        },
        action: async (ctx) => {
            await installResourceAction(ctx, ResourceType.WatchFace, filePath);
        },
    };
}

// third-party app任务
export function createThirdPartyAppInstallTask(id: string, name: string, filePath: string): TaskItem {
    return {
        id,
        name,
        description: "installTask.thirdPartyApp.description",
        type: ResourceType.ThirdPartyApp,
        icon: AppsRegular, // 替换为实际的图标组件
        payload: {
            url: filePath,
            progress: 0,
            status: "pending",
        },
        action: async (ctx) => {
            await installResourceAction(ctx, ResourceType.ThirdPartyApp, filePath);
        },
    };
}

// firmware任务
export function createFirmwareInstallTask(id: string, name: string, filePath: string): TaskItem {
    return {
        id,
        name,
        description: "installTask.firmware.description",
        type: ResourceType.Firmware,
        icon: BoxMultipleRegular, // 替换为实际的图标组件
        payload: {
            url: filePath,
            progress: 0,
            status: "pending",
        },
        action: async (ctx) => {
            await installResourceAction(ctx, ResourceType.Firmware, filePath);
        },
    };
}

export async function installResourceAction(ctx: TaskActionContext, type: ResourceType, filePath: string) {
    const cb = new Channel<SendMassCallBackData>();
    const finished = new Promise<void>(resolve => {
        cb.onmessage = (data) => {
            requestAnimationFrame(() => {
                ctx.update({
                    payload: {
                        url:filePath,
                        progress: data.progress,
                        status: `running`
                    }
                });
            });
            if (data.progress === 1) resolve();
        };
    });

    let cmd = "";
    let params: any = { filePath, onProgress: cb };

    switch (type) {
        case ResourceType.ThirdPartyApp:
            cmd = "miwear_install_third_app";
            params.packageName = "";
            params.versionCode = 1;
            break;
        case ResourceType.WatchFace:
            //@ts-ignore
            if(ctx.get()?.newWatchfaceID) {
                const idBytes = new Uint8Array(12);
                //@ts-ignore
                const inputBytes = new TextEncoder().encode(ctx.get()?.newWatchfaceID);
                idBytes.set(inputBytes.subarray(0, 12))
                params.newWatchfaceId = idBytes
            }else {
                params.newWatchfaceId = null;
            }
            cmd = "miwear_install_watchface";
            break;
        case ResourceType.Firmware:
            cmd = "miwear_install_firmware";
            break;
    }
    await invoke(cmd, params);
    await finished;
    ctx.update({
        payload: {
            url: filePath,
            progress: 1,
            status: "success",
        }
    });
}