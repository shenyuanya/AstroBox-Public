import { accountManager } from "@/account/manager";
import { Account } from "@/account/provider";
import AccountCard from "@/components/AccountCard/AccountCard";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import BasePage from "@/layout/basePage";
import SettingsGroup from "@/layout/settingsGroup";
import { useI18n } from "@/i18n";
import { Avatar, Body1Strong, CardHeader, Link, Title3 } from "@fluentui/react-components";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
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
        {/* <PurchasedRes /> */}
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
    return <SettingsGroup title={t('bandbbs.purchased')}>
        <AccountCard
            avatar="AurysianYan"
            content="Resource01"
            secondaryContent={t('bandbbs.purchasedDescription')}
        />
    </SettingsGroup>
}