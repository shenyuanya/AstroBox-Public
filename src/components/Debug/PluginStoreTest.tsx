import { useI18n } from "@/i18n";
import { ProgressData } from "@/plugin/types";
import { providerManager } from "@/pluginstore/manager";
import { Button, Label } from "@fluentui/react-components";
import { Channel } from "@tauri-apps/api/core";
import { useState } from "react";

export default function PluginStoreTest() {
    const { providers } = providerManager.useProviders();
    const { t } = useI18n();
    const [execRet, setExecRet] = useState(t('pluginStoreTest.waiting'));

    const getProvs = () => {
        let ret: string[] = []
        providers.forEach(prov => {
            ret.push(prov.name);
        })
        
        setExecRet(JSON.stringify(ret));
    }

    const refAll = async() => {
        await providerManager.refreshAll();
        setExecRet(t('pluginStoreTest.success'));
    }

    const pp = async() => {
        setExecRet(JSON.stringify(await providerManager.get("official")?.getPage(0, 100)));
    }

    const gitest = async() => {
        setExecRet(JSON.stringify(await providerManager.get("official")?.getItem("测试插件")));
    }

    const downtest = async() => {
        const cb = new Channel<ProgressData>();
        cb.onmessage = (data) => {
            setExecRet(JSON.stringify(data))
        }
        setExecRet(JSON.stringify(await providerManager.get("official")?.download("测试插件", "", cb)));
    }

    return (
        <div>
            <Button onClick={getProvs}>providers</Button>
            <Button onClick={refAll}>{t('pluginStoreTest.refreshAll')}</Button>
            <Button onClick={pp}>{t('pluginStoreTest.pullPage')}</Button>
            <Button onClick={gitest}>{t('pluginStoreTest.pullItem')}</Button>
            <Button onClick={downtest}>{t('pluginStoreTest.downloadAndPackage')}</Button>
            <Label>{execRet}</Label>
        </div>
    )
}