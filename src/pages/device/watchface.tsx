import BasePage from "@/layout/basePage";
import {WatchfaceInfo} from "@/types/bluetooth";
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
import {MoreHorizontalRegular} from "@fluentui/react-icons";
import {invoke} from "@tauri-apps/api/core";
import {AnimatePresence, motion} from "framer-motion";
import {useInvokeWithMass, useIsSendingMass} from "@/hooks/useInvoke";
import {useMemo} from "react";
import { useI18n } from "@/i18n";

export default function Watchface() {
    const { t } = useI18n();
    const fuckWebkit = useMemo(() => localStorage.getItem("fkWebkit") === "true", []);
    const {isSendingMass, mutate: m1} = useIsSendingMass();
    const {
        data: watchfaceList,
        mutate: m2
    } = useInvokeWithMass<WatchfaceInfo[]>(isSendingMass)("miwear_get_watchface_list")
    const refresh = () => {
        m1();
        m2()
    }
    return <BasePage title={t('watchfaceManagement.title')}>
        <div style={{ gap: 5, display: "flex", flexDirection: "column", marginTop: 5 }}>
            <AnimatePresence>
                {(!watchfaceList || !watchfaceList.length) ? (
                <div style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
                    <Body1>{t('watchfaceManagement.none')}</Body1>
                </div>) : watchfaceList.map((watchface, index) => <motion.div
                    transition={{
                        default: {
                            duration: 0.3,
                            delay: index * 0.05,
                        }
                    }}
                    initial={{opacity: fuckWebkit ? 1 : 0, transform: fuckWebkit ? "" : "translateY(20px)"}}
                    animate={{ opacity: 1, transform: "translateY(0)" }}
                    exit={{ opacity: 0 }}
                    className="card">
                    <AppInfoCard watchface={watchface} refresh={refresh} />
                </motion.div>)
                }
            </AnimatePresence>
        </div>
    </BasePage>;
}
function AppInfoCard({ watchface, refresh }: { watchface: WatchfaceInfo, refresh: () => void }) {
    return <CardHeader
        header={<Body1>{watchface.name}</Body1>}
        description={<Caption1>{watchface.id}</Caption1>}
        action={<AppActions watchface={watchface} refresh={refresh} />}

    ></CardHeader>
}
function AppActions({ watchface, refresh }: { watchface: WatchfaceInfo, refresh: () => void }) {
    const { t } = useI18n();

    return <Menu>
        <MenuTrigger>
            <Button appearance="subtle" icon={<MoreHorizontalRegular />}></Button>
        </MenuTrigger>
        <MenuPopover>
            <MenuList>
                <MenuItem onClick={() => {
                    invoke("miwear_set_watchface", { watchface }).finally(() => {})
                }}>{t('common.enable')}</MenuItem>
                <MenuItem
                    disabled={!watchface.can_remove}
                    onClick={() => {
                        invoke("miwear_uninstall_watchface", { watchface }).finally(() => {
                            setTimeout(() => {
                                refresh();
                            }, 500);
                        })
                    }}>{t('common.uninstall')}</MenuItem>
            </MenuList>
        </MenuPopover>
    </Menu>
}