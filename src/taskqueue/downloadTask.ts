import { providerManager } from "@/community/manager";
import { getFileType, installABP, ResourceType } from "@/device/install";
import logger from "@/log/logger";
import { ProgressData } from "@/plugin/types";
import { providerManager as abpProviderMgr } from "@/pluginstore/manager";
import { Channel, invoke } from "@tauri-apps/api/core";
import { installResourceAction } from "./installTask";
import { installList } from "./queue";
import { TaskActionContext, TaskItem } from "./tasklist";
import { getFilenameFromPath } from "@/filesystem/pathhelper";
import { BaseDirectory, remove } from "@tauri-apps/plugin-fs";

// 社区文件任务
export function createDownloadTask(id: string, name: string, provider: string, device: string, description: string, icon: React.ComponentType<any>): TaskItem {
    return {
        id,
        name,
        description,
        icon,
        payload: {
            url: "",
            progress: 0,
            status: "pending",
        },
        action: async (ctx: TaskActionContext) => {
            await downloadResourceAction(ctx, id, provider, device);

            let respath = ctx.get()?.payload?.url as string;
            console.log(ctx.get())
            let restype = await getFileType(respath);
            logger.info(`online resource downloaded. filePath=${respath} resType=${restype}`);

            if (restype === null || !respath) throw new Error("download file failed");

            if (restype == ResourceType.ABP) return installABP(respath)
            installList.add({
                id: id,
                name: name,
                type: restype,
                description: description,
                icon: icon,
                payload: {
                    url: respath,
                    progress: 0,
                    status: "pending",
                },
                action: async (ctx: TaskActionContext) => {
                    await installResourceAction(ctx, restype, respath);
                    await remove(`tmp/${getFilenameFromPath(respath)}`, { baseDir: BaseDirectory.AppCache });
                },
            })
            const config = await invoke<any>("app_get_config");
            if (config.auto_install) {
                if(installList.status === "pending"){
                    installList.run();
                }
            }
        }
    }
}

async function downloadResourceAction(ctx: TaskActionContext, id: string, provider: string, device: string) {
    const cb = new Channel<ProgressData>();
    cb.onmessage = (data) => {
        requestAnimationFrame(() => {
            ctx.update({
                payload: {
                    url: "",
                    progress: data.progress,
                    status: "running"
                }
            });
        });
    }

    let prov = device == "abp" ? abpProviderMgr.get(provider) : providerManager.get(provider);

    //@ts-ignore
    let filePath = await prov?.download(id, device, cb);

    if (filePath) {
        ctx.update({
            payload: {
                url: filePath,
                progress: 0,
                status: "running"
            }
        })
    }
    else {
        throw new Error("我文件呢?");
    }
}