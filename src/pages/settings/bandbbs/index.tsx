import { accountManager } from "@/account/manager";
import { Account } from "@/account/provider";
import { providerManager } from "@/community/manager";
import CardButton from "@/components/CardButton/CardButton";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import { useI18n } from "@/i18n";
import BasePage from "@/layout/basePage";
import SettingsGroup from "@/layout/settingsGroup";
import { createDownloadTask } from "@/taskqueue/downloadTask";
import { addDownloadTask } from "@/taskqueue/queue";
import { ResourceManifestV1 } from "@/types/ResManifestV1";
import { Avatar, Body1Strong, Button, CardHeader, Input, Label, Link, Title3 } from "@fluentui/react-components";
import { AppsRegular, SearchRegular } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import Image from "next/image";
import { useEffect, useRef, useState } from "react";
import styles from "./accountInfo.module.css";

export default function Bandbbs() {
    const [bandbbsAccount, setBandbbsAccount] = useState<Account>()
    const router = useAnimatedRouter()
    const { t } = useI18n();
    useEffect(() => {
        accountManager.refresh().then(() => accountManager.get("bandbbs")?.list())
            .then(res => setBandbbsAccount(res?.[0]))
    }, [])
    return <BasePage title={t('bandbbs.title')}>
        <div className={styles.accountInfo}>
            <CardHeader
                image={<Avatar name={bandbbsAccount?.username} image={{src:bandbbsAccount?.avatar}} color="colorful" size={72} style={{ marginLeft: "-2px", marginRight: "4px" }}></Avatar>}
                header={{
                    children: <><Title3>{bandbbsAccount?.username}</Title3><Body1Strong className={styles.caption}>{t('bandbbs.uid')}{bandbbsAccount?.id}</Body1Strong></>,
                    className: styles.headerSlot,
                }}
            />
        </div>
        <PurchasedRes />
        <Link href="" onClick={()=>{
            invoke("account_logout",{
                name:"bandbbs",
                account:bandbbsAccount
            }).then(res=>{
                setBandbbsAccount(undefined)
                router.back()
            })
        }} style={{ width: "100%", textAlign: "center",color:"var(--colorStatusDangerForeground3)" }}>
            {t('bandbbs.logout')}
        </Link>
    </BasePage>
}
function PurchasedRes() {
    const { t } = useI18n();
    const { providers } = providerManager.useProviders();
    const bandbbsProvider = providers.find(provider => provider.name === "bandbbs");
    const [item, setItem] = useState<ResourceManifestV1>();
    const input = useRef<HTMLInputElement>(null);

    const downloads = []
    if (item) {
        for (const key in item.downloads) {
            downloads.push(key)
        }
    }
    const download = async (code: string) => {
        const { icon, name, _bandbbs_ext_resource_id, description } = item?.item ?? {};
        const iconComponent = () => icon ? <Image width={40} height={40} src={icon} alt={name ?? ""} style={{ borderRadius: 999 }} /> : <AppsRegular />;
        const taskId = _bandbbs_ext_resource_id?.toString() ?? name ?? "";
        const taskName = name ?? "";
        const displayDescription = code;
        const task = createDownloadTask(
            taskId,
            taskName,
            "bandbbs",
            code,
            displayDescription,
            iconComponent
        );
        addDownloadTask(task);
    }
    return <SettingsGroup title={t('bandbbs.purchased')}>
        <div className="card" style={{ flexDirection: "column", display: "flex", gap: 5 }}>

            <Label>{t('bandbbs.enter_resource_id')}</Label>
            <div style={{ display: 'flex', flexDirection: 'row', gap: 5 }}>
                <Input placeholder={t('bandbbs.resource_id_placeholder')} ref={input} style={{ flex: 6 }} type='number' min={1} inputMode='numeric' />
                <Button appearance='primary' style={{ flex: 1 }}
                    icon={<SearchRegular />}
                    onClick={() => {
                        bandbbsProvider?.getItem(input.current?.value ?? '').then(res => {
                            setItem(res)
                        })
                    }}>{t('bandbbs.query')}</Button>
            </div>


            {downloads.map((key, index) => (
                <CardButton content={key} key={index} className={styles.resCard} onClick={() => download(key)}></CardButton>
            ))}
        </div>
    </SettingsGroup>
}