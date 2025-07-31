import { installResourceFromLocal, ResourceType } from "@/device/install";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import useIsMobile from "@/hooks/useIsMobile";
import { useI18n } from "@/i18n";
import BasePage from "@/layout/basePage";
import { PluginManifest } from "@/plugin/types";
import {
    Button,
    Field,
    SearchBox,
    SelectTabEventHandler,
    Tab,
    TabList,
    TeachingPopover,
    TeachingPopoverBody,
    TeachingPopoverFooter,
    TeachingPopoverSurface,
    TeachingPopoverTitle,
    TeachingPopoverTrigger,
    tokens
} from "@fluentui/react-components";
import { AppsAddInFilled, WarningFilled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useState } from "react";
import PluginInstalled from "../../components/plugin/pluginInstalled";
import PluginStore from "../../components/plugin/pluginStore";
import styles from "./plugin.module.css";
import PluginInfo from "./pluginInfo";

export default function Plugin() {
    const { t } = useI18n();
    const [tab, setTab] = useState("installed");
    const [infoShow, setInfoShow] = useState<PluginManifest | null>(null);
    const [local, setLocal] = useState(false);
    const [search, setSearch] = useState<string>("");
    const router = useAnimatedRouter()
    const isMobile = useIsMobile()
    useEffect(() => {
        if (isMobile) {
            showInfo(infoShow);
        }
        setTab((window.location.hash as string || "#installed").slice(1));
    }, [isMobile])
    const handleTabSelect: SelectTabEventHandler = (e, data) => {
        setTab(data.value as string);
        // 设置返回时的路径
        window.location.hash = data.value as string;
    }
    const showInfo = (plugin: PluginManifest | null, local1: boolean = local) => {
        if (isMobile && plugin) {
            router.push("/plugin/pluginInfo?plugin=" + encodeURIComponent(JSON.stringify(plugin)) + "&local=" + local1);
            setInfoShow(null);
            return;
        }
        setInfoShow(plugin);
        setLocal(local1);
    }
    return (
        <div style={{ height: "100%", display: "flex", flexDirection: "row" }}>
            <BasePage title={t('nav.plugin')} style={{ flex: 1 }} action={
                <Field><SearchBox placeholder={t('plugin.searchPlaceholder')} onChange={(e, data) => {
                    //@ts-ignore
                    if (!e.nativeEvent.isComposing) setSearch(data.value)
                }}
                    onCompositionEnd={(e) => { setSearch(e.data) }}
                /></Field>
            }>
                <TabList selectedValue={tab} size="small" className={styles.tabbar}
                    onTabSelect={handleTabSelect}
                >
                    <Tab value="installed">{t('plugin.tabs.installed')}</Tab>
                    <Tab value="market">{t('plugin.tabs.market')}</Tab>
                    <div style={{ flex: 2 }} />
                    <Button appearance="transparent" icon={<AppsAddInFilled />}
                        style={{ color: tokens.colorBrandForeground2Hover, textWrap: "nowrap" }}
                        onClick={() => { installResourceFromLocal(ResourceType.ABP) }}
                    >
                        {t('plugin.import')}
                    </Button>
                    <RestartTtigger />
                </TabList>
                {tab === "market" && <PluginStore onSelect={(plugin) => {
                    showInfo(plugin, false)
                }} search={search}/>}
                {tab === "installed" && <PluginInstalled onSelect={(plugin) => { showInfo(plugin, true) }} search={search} />}
            </BasePage>
            <AnimatePresence mode="popLayout">
                {infoShow && <motion.div
                    layout
                    initial={{ x: 100, opacity: 0 }}
                    animate={{ x: 0, opacity: 1 }}
                    exit={{ x: 100, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    style={{ flex: 1, maxWidth: "50vw", background: "var(--cardbackground)", marginTop: "0px", marginLeft: "10px", marginRight: "-16px", marginBottom: "-16px", padding: "0 10px", borderRadius: "var(--border-radius)" }}><PluginInfo plugin={infoShow!} local={local} onExit={() => { showInfo(null) }} /> </motion.div>}
            </AnimatePresence>
        </div>
    )
}
function RestartTtigger() {
    const { t } = useI18n();
    const [show, setShow] = useState(false);
    useEffect(() => {
        const getUpdated = () => invoke<boolean>("plugsys_is_updated").then((res) => {
            if (res) clearInterval(interval);
            return res
        }).then(setShow)
        getUpdated()
        const interval = setInterval(getUpdated, 1000)
        return () => clearInterval(interval);
    })
    if (!show) return <></>
    return (<TeachingPopover>
        <TeachingPopoverTrigger disableButtonEnhancement>
            <Button appearance="primary" icon={<WarningFilled />} style={{ background: tokens.colorStatusDangerBackground3, textWrap: "nowrap" }}>
                {t('plugin.restartPrompt.title')}
            </Button>
        </TeachingPopoverTrigger>
        <TeachingPopoverSurface>
            <TeachingPopoverTitle>{t('plugin.restartPrompt.title')}</TeachingPopoverTitle>
            <TeachingPopoverBody>
                {t('plugin.restartPrompt.body')}
            </TeachingPopoverBody>
            <TeachingPopoverFooter primary={{
                style: { background: tokens.colorStatusDangerBackground3 }, children: t('plugin.restartPrompt.primary'), onClick: () => {
                    invoke("cleanup_before_exit")
                }
            }} secondary={t('plugin.restartPrompt.secondary')} />
        </TeachingPopoverSurface>
    </TeachingPopover>)
}