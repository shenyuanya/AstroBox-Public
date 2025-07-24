import PluginCard from "@/components/plugincard/PluginCard";
import {PluginManifest, PluginState} from "@/plugin/types";
import {invoke} from "@tauri-apps/api/core";
import {useEffect, useState} from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import {FixedSizeList} from 'react-window';
import {Body1} from "@fluentui/react-components";
import { useI18n } from "@/i18n";

export default function PluginInstalled({onSelect,search}:{onSelect:(plugin:PluginManifest|null)=>void,search?:string}) {

    const { t } = useI18n();

    const [plugins, setPlugins] = useState<PluginManifest[]>([])
    useEffect(()=>{
        invoke<PluginManifest[]>("plugsys_get_list").then((res) => Promise.all(
            res.filter((plugin: PluginManifest) => {
                if(!plugin) return false
                return plugin?.name?.toLowerCase()?.includes(search?.toLowerCase()??"")
            }).map(async (e) => {
                const { disabled, icon_b64 } = await invoke<PluginState>("plugsys_get_state", { name: e.name })
                return {
                    ...e,
                    icon: icon_b64 ? icon_b64 : e.icon,
                    disabled,
                }
            })
        )).then(e=>{
            setPlugins(e)
        })
    },[search])
    const Item = ({ index, style }: any) => {
        style = {
            ...style,
            width: "calc(100% - 20px)",
        }
        return (
            <div style={style}>
                <PluginCard plugin={plugins[index]} local onClick={()=>{onSelect(plugins[index])}} />
            </div>
        )
    }
    return (
        <div style={{flex:1}}>
            {plugins.length?<AutoSizer>
                {({ height, width }) =>
                    <FixedSizeList height={height} width={width}
                        itemCount={plugins.length}
                        itemSize={80}
                        style={{ overflow:"visible"}}
                    >
                        {Item}
                    </FixedSizeList>
                }
            </AutoSizer> : <Body1 style={{width: "100%", textAlign: "center", margin: "auto"}}>{t('plugin.noPluginsInstalled')}</Body1>}
        </div>
    )
}