import { pickFile } from "@/filesystem/picker";
import logger from "@/log/logger";
import { createFirmwareInstallTask, createThirdPartyAppInstallTask, createWatchFaceInstallTask } from "@/taskqueue/installTask";
import { addInstallTask } from "@/taskqueue/queue";
import { invoke } from "@tauri-apps/api/core";
import { listen } from '@tauri-apps/api/event';
import { basename } from "@tauri-apps/api/path";
import { platform } from "@tauri-apps/plugin-os";
import { getFilenameFromPath } from "../filesystem/pathhelper";

export enum ResourceType {
    WatchFace,
    ThirdPartyApp,
    Firmware,
    ABP,
}

async function getPickerConfig(type: ResourceType) {
    const curplatform = await platform();
    if(curplatform ==="android" || curplatform ==="ios")return null;
    switch (type) {
        case ResourceType.WatchFace:
            return { name: "WatchFace Files", extensions: ["bin", "face"] };
        case ResourceType.Firmware:
            return { name: "Firmware Files", extensions: ["bin", "zip"] };
        case ResourceType.ABP:
            return { name: "AstroBox Plugin", extensions: ["abp"] }
        default:
            return { name: "QuickApp Packages", extensions: ["rpk"] };
    }
}


export async function installResourceFromLocal(type: ResourceType) {
    const pickerConfig = await getPickerConfig(type);
    //@ts-ignore
    const file = await pickFile(true, pickerConfig ? [pickerConfig] : undefined);
    if (!file || file === "") return;

    const fileList = Array.isArray(file) ? file : [file];

    fileList.forEach(f => {
        let task;
        const filename = getFilenameFromPath(f) ?? "";
        switch (type) {
            case ResourceType.WatchFace:
                task = createWatchFaceInstallTask(f, filename, f);
                break;
            case ResourceType.ThirdPartyApp:
                task = createThirdPartyAppInstallTask(f, filename, f);
                break;
            case ResourceType.Firmware:
                task = createFirmwareInstallTask(f, filename, f);
                break;
            case ResourceType.ABP:
                return installABP(f)
            default:
                return;
        }
        addInstallTask(task);
    });
}

export async function installfile(path: string) {
    const filetype = await getFileType(path)
    if(filetype === ResourceType.ABP) return installABP(path)
    if (filetype !== null) {
        let task;
        const filename = getFilenameFromPath(path) ?? "";
        switch (filetype) {
            case ResourceType.WatchFace:
                task = createWatchFaceInstallTask(path, filename, path);
                break;
            case ResourceType.ThirdPartyApp:
                task = createThirdPartyAppInstallTask(path, filename, path);
                break;
            default:
                return;
        }
        addInstallTask(task);
    }
}
export async function getFileType(path:string){
    const filetype = await invoke<string>("get_file_type",{path})
    return getType(filetype)
}
type fileOpened = {
    path:string;
    file_type:string;
}
export async function registerOpenFileListener() {
    return await listen<fileOpened>("open_file",(event)=>{
        try {
            const type = getType(event.payload.file_type)
            if (type === ResourceType.ABP) return installABP(event.payload.path)
            if (type !== null) {
                let task;
                const filename = getFilenameFromPath(event.payload.path) ?? "";
                switch (type) {
                    case ResourceType.WatchFace:
                        task = createWatchFaceInstallTask(event.payload.path, filename, event.payload.path);
                        break;
                    case ResourceType.ThirdPartyApp:
                        task = createThirdPartyAppInstallTask(event.payload.path, filename, event.payload.path);
                        break;
                    default:
                        return;
                }
                addInstallTask(task);
            }
        }catch (e){
            logger.error(e)
        }
    })
}
function getType(type:string){
    switch (type){
        case "quickapp":
            return ResourceType.ThirdPartyApp
        case "watchface":
            return ResourceType.WatchFace
        case "abp":
            return ResourceType.ABP
        default:
            return null
    }
}
export async function installABP(path: string) {
    const name = await basename(path, ".abp")
    await invoke("plugsys_install_abp",{
        name,
        path
    })
}