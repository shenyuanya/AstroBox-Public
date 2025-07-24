import PluginNodeUI from "@/components/PluginNodeUI/PluginNodeUI";
import { PluginManifest, PluginUINode } from "@/plugin/types";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import emptyIllustration from '@/components/illustration/empty.svg';
import { useI18n } from '@/i18n';

export default function PluginFeatures({ plugin }: { plugin: PluginManifest }) {
    const { t } = useI18n();
    const [settingsUINodes, setSettingsUINodes] = useState<PluginUINode[]>([])

    const updateSettingsUI = () => {
        invoke<PluginUINode[]>("plugsys_get_settings_ui_nodes", { name: plugin.name }).then((res) => {
            setSettingsUINodes(res)
        })
    }
    const onSettingsUICallback = async (id: string, value: string = "") => {
        await invoke("plugsys_call_registered_func", {
            name: plugin.name,
            funcId: id,
            payload: value
        })
    }
    useEffect(() => {
        let unlisten = () => { }
        updateSettingsUI()
        listen("plugin-update-settings-page", () => {
            updateSettingsUI()
        }).then((res) => {
            unlisten = res
        })
        return () => {
            unlisten?.()
        }
    }, [plugin])

    return (
        <>
            {settingsUINodes.length > 0 ? (
                <div style={{ marginTop: 20 }}>
                    <PluginNodeUI nodes={settingsUINodes} onCallback={onSettingsUICallback} />
                </div>
            ) : (
                <div style={{ width: "100%", display: "flex", flexDirection: "column", alignItems: "center" }}>
                    <img src={emptyIllustration.src} alt="empty" width={200} height={150} />
                    <div>{t('plugin.noFeatures')}</div>
                </div>
            )}
        </>
    )
}