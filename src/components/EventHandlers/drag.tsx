import { installfile, installResourceFromLocal, ResourceType } from "@/device/install";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import { useI18n } from "@/i18n";
import { Body1Strong, Dialog, DialogBody, DialogSurface, Subtitle1 } from "@fluentui/react-components";
import { DragRegular } from '@fluentui/react-icons';
import { listen, TauriEvent, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

export default function DragToPush() {
    const router = useAnimatedRouter();
    const [dragging, setDragging] = useState(false);
    const { t } = useI18n();

    useEffect(() => {
        // 监听拖拽事件
        const unlisten: Promise<UnlistenFn>[] = []
        unlisten.push(listen(TauriEvent.DRAG_OVER, async () => {
            setDragging(true);
        }))
        unlisten.push(listen(TauriEvent.DRAG_LEAVE, async () => {
            setDragging(false);
        }))
        unlisten.push(listen<{ paths: string[] }>(TauriEvent.DRAG_DROP, async event => {
            setDragging(false);
            event.payload.paths.forEach((e: string) => {
                installfile(e);
            })
        }))

        // 监听menu事件
        unlisten.push(listen("menubar_switch_device", () => {
            router.push("device/switch")
        }))
        unlisten.push(listen("menubar_open_rpk", () => {
            installResourceFromLocal(ResourceType.ThirdPartyApp)
        }))
        unlisten.push(listen("menubar_open_wf", () => {
            installResourceFromLocal(ResourceType.WatchFace)
        }))
        unlisten.push(listen("menubar_open_fw", () => {
            installResourceFromLocal(ResourceType.Firmware)
        }))
        unlisten.push(listen("menubar_about", () => {
            router.push("my/about/about")
        }))

        return () => {
            unlisten.forEach(e => e.then(fn => fn()));
        }
    })

    return (
        <Dialog open={dragging}>
            <DialogSurface style={{ background: "transparent", boxShadow: "none !important", border: "2px dashed var(--colorNeutralForeground4)", maxWidth: "480px" }}>
                <DialogBody style={{
                    display: "flex", flexDirection: "column", alignItems: "center",
                    borderRadius: "6px",
                    padding: "48px",
                }}>
                    <DragRegular style={{ fontSize: "48px" }} />
                    <Subtitle1>{t('dragToPush.title')}</Subtitle1>
                    <Body1Strong>{t('dragToPush.supportedFiles')}</Body1Strong>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    )
}