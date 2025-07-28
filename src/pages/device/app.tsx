import OpenToPageDialog from "@/components/OpenToPageDialog/OpenToPageDialog";
import { useInvokeWithMass, useIsSendingMass } from "@/hooks/useInvoke";
import { useI18n } from "@/i18n";
import BasePage from "@/layout/basePage";
import useToast, { makeError } from "@/layout/toast";
import { AppInfo } from "@/types/bluetooth";
import {
    Body1,
    Button,
    Caption1,
    CardHeader,
    Menu,
    MenuItem,
    MenuList,
    MenuPopover,
    MenuTrigger
} from "@fluentui/react-components";
import { MoreHorizontalRegular } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion } from "framer-motion";
import { useMemo } from "react";

export default function App() {
    const {isSendingMass, mutate: m1} = useIsSendingMass();
    const {data: appList, mutate: m2} = useInvokeWithMass<AppInfo[]>(isSendingMass)("miwear_get_app_list")
    const fuckWebkit = useMemo(() => localStorage.getItem("fkWebkit") === "true", []);
    const { t } = useI18n();
    const refresh = () => {
        m1().finally(() => m2())
    }
    return <BasePage title={t('appManagement.title')}>
        <div style={{ gap: 5, display: "flex", flexDirection: "column", marginTop: 5 }}>
            <AnimatePresence>
                {!appList || !appList.length ? (
                <div style={{display:"flex",flexDirection:"column",alignItems:"center"}}>
                    <Body1>{t('appManagement.none')}</Body1>
                </div>
                ) : appList.map((app, index) => <motion.div
                    transition={{
                        default: {
                            duration: 0.3,
                            delay: index * 0.05,
                        }
                    }}
                    initial={{opacity: fuckWebkit ? "" : 1, transform: fuckWebkit ? "" : "translateY(20px)"}}
                    animate={{opacity: 1, transform: fuckWebkit ? "" : "translateY(0)"}}
                    exit={{opacity: 0}}
                    className="card">
                    <AppInfoCard app={app} refresh={refresh}/>

                </motion.div>)}
            </AnimatePresence>

        </div>
    </BasePage>;
}

function AppInfoCard({app, refresh}: { app: AppInfo, refresh: () => void }) {

    return <CardHeader
            header={<Body1>{app.app_name}</Body1>}
            description={<Caption1>{app.package_name}</Caption1>}
            action={<AppActions app={app} refresh={refresh}/>}
        ></CardHeader>
}

function AppActions({app, refresh}: { app: AppInfo, refresh: () => void }) {
    const { t } = useI18n();
    const { dispatchToast } = useToast();

    return <Menu>
        <MenuTrigger>
            <Button appearance="subtle" icon={<MoreHorizontalRegular />}></Button>
        </MenuTrigger>
        <MenuPopover>
            <MenuList>
                <MenuItem onClick={()=>{
                    invoke("miwear_open_quickapp",{app,page:""})
                }}>{t('common.open')}</MenuItem>
                <MenuItem onClick={()=>{
                    invoke("miwear_uninstall_quickapp", { app }).catch(() => { makeError(dispatchToast, t('common.fail')) }).finally(() => {
                        setTimeout(() => {
                            refresh();
                        }, 500);
                    })
                }}>{t('common.uninstall')}</MenuItem>
                <OpenToPageDialog app={app}/>
            </MenuList>
        </MenuPopover>
    </Menu>
}