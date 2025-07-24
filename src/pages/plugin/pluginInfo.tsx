import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import { useI18n } from "@/i18n";
import BasePage from "@/layout/basePage";
import SettingsGroup from "@/layout/settingsGroup";
import { PluginManifest, PluginState } from "@/plugin/types";
import { createDownloadTask } from "@/taskqueue/downloadTask";
import { addDownloadTask } from "@/taskqueue/queue";
import {
    Body1,
    Body1Strong,
    Button,
    Link,
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
import {
    AppsAddInFilled,
    DismissRegular,
    FullScreenMaximizeRegular,
    GlobeRegular,
    InfoRegular,
    NumberSymbolRegular,
    OpenRegular,
    PersonRegular,
    PuzzlePiece48Regular,
    ShieldKeyholeRegular,
    UninstallAppFilled
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { openUrl } from "@tauri-apps/plugin-opener";
import Image from "next/image";
import { useEffect, useState } from "react";
import PluginFeatures from "../../components/plugin/pluginFeatures";
import styles from "./plugin.module.css";

export default function PluginInfo({ plugin, local, onExit }: { plugin?: PluginManifest, local?: boolean, onExit?: () => void }) {
    const router = useAnimatedRouter()
    const { t } = useI18n();

    if (!plugin) {
        if (typeof router.query.plugin !== 'string' || typeof router.query.local !== 'string') return <BasePage title={t('plugin.detailTitle')}></BasePage>
        plugin = JSON.parse(router.query.plugin) as PluginManifest
        local = JSON.parse(router.query.local) as boolean
    }

    const [icon, setIcon] = useState(plugin.icon)
    const [disabled, setDisabled] = useState(plugin.disabled)
    const [page, setPage] = useState(local ? "features" : "doc")
    const [isInstalled, setIsInstalled] = useState(false);

    useEffect(() => {
        if (local) {
            invoke<PluginState>("plugsys_get_state", { name: plugin.name }).then((res) => {
                setIcon(res.icon_b64);
                setDisabled(res.disabled);
            })
        } else {
            setIcon(plugin.icon)
        }
    }, [plugin, local])

    useEffect(() => {
        if (!local && plugin?.name) {
            setIsInstalled(false);
            invoke<PluginManifest[]>("plugsys_get_list").then((installedList) => {
                if (installedList.some(p => p.name === plugin.name)) {
                    setIsInstalled(true);
                }
            });
        }
    }, [plugin, local]);

    const handleDisable = () => {
        const command = disabled ? "plugsys_enable" : "plugsys_disable";
        invoke(command, { name: plugin.name }).then(() => {
            setDisabled(!disabled);
        })
    }

    const handleDownload = () => {
        const iconComponent = () => icon ? <Image src={icon} alt={plugin.name ?? ""} width={40} height={40}
            style={{ gridRowStart: "span 2", gridColumnStart: 1 }} /> :
            <PuzzlePiece48Regular />
        const task = createDownloadTask(plugin.name ?? "", plugin.name ?? "", "official", "abp", plugin.description ?? "", iconComponent)
        addDownloadTask(task);
    }

    return (
        <BasePage title={t('plugin.detailTitle')} action={
            (onExit && <div style={{ gap: "10px" }}>
                <AppleButtonWrapper>
                    <Button appearance="transparent"
                        icon={<FullScreenMaximizeRegular />}
                        onClick={() => { router.push("/plugin/pluginInfo?plugin=" + encodeURIComponent(JSON.stringify(plugin)) + "&local=" + local); }} />
                </AppleButtonWrapper>
                <AppleButtonWrapper>
                    <Button appearance="transparent"
                        icon={<DismissRegular />}
                        onClick={onExit} />
                </AppleButtonWrapper>
            </div>)
        }>
            <div className={styles.pluginInfo}>
                {icon ? <Image src={icon} alt={plugin.name ?? ""} width={72} height={72} style={{ gridRowStart: "span 2", gridColumnStart: 1, borderRadius: tokens.borderRadiusMedium }} /> : <PuzzlePiece48Regular />}
                <div className={styles.pluginInfoContent}>
                    <h2 style={{ width: "100%", fontSize: "22px", margin: "0", fontWeight: "var(--fontWeightMedium)" }}>{plugin.name}</h2>
                    <p style={{ margin: "0", opacity: 0.75 }}>{plugin.description}</p>
                </div>
                <div className={styles.pluginInfoAction}>
                    {local ? <>
                        <TeachingPopover>
                            <TeachingPopoverTrigger disableButtonEnhancement>
                                <Button appearance="outline" icon={<UninstallAppFilled />} style={{ color: tokens.colorStatusDangerForeground2, borderColor: "color-mix(in srgb, var(--colorStatusDangerForeground2) 40%, transparent)" }}>
                                    {t('plugin.uninstallPrompt.primary')}
                                </Button>
                            </TeachingPopoverTrigger>
                            <TeachingPopoverSurface>
                                <TeachingPopoverBody>
                                    <TeachingPopoverTitle>{t('plugin.uninstallPrompt.title')}</TeachingPopoverTitle>
                                    <div>{t('plugin.uninstallPrompt.body').replace('{pluginName}', plugin.name ?? '')}</div>
                                </TeachingPopoverBody>
                                <TeachingPopoverFooter primary={{
                                    style: { background: tokens.colorStatusDangerBackground3 }, children: t('plugin.uninstallPrompt.primary'), onClick: () => { invoke("plugsys_remove", { name: plugin.name }).finally(() => { onExit?.() }) }
                                }} secondary={t('plugin.uninstallPrompt.secondary')} />
                            </TeachingPopoverSurface>
                        </TeachingPopover>
                        <Button onClick={handleDisable} appearance={disabled ? "primary" : "outline"}>{disabled ? t('common.enable') : t('common.disable')}</Button>
                    </> : <Button icon={<AppsAddInFilled />} appearance="primary" onClick={handleDownload} disabled={isInstalled}>{isInstalled ? t('plugin.tabs.installed') : t('common.install')}</Button>}
                </div>
            </div>
            {
                local && <TabList size="small" selectedValue={page} onTabSelect={(e, data) => { setPage(data.value as string) }}>
                    <Tab value="features">{t('plugin.tabs.features')}</Tab>
                    <Tab value="doc">{t('plugin.tabs.details')}</Tab>
                </TabList>
            }
            <div style={{ margin: "10px 2px" }}>
                {page === "features" && <PluginFeatures plugin={plugin} />}
                {page === "doc" && <PluginDoc plugin={plugin} />}
            </div>

        </BasePage >
    )
}

function InfoLine({ icon, label, children }: { icon: React.ReactNode, label: string, children: React.ReactNode }) {
    return (
        <div className={styles.infoLine}>
            <div className={styles.infoLineLabel}>
                {icon}
                <Body1 style={{ whiteSpace: "nowrap", textWrap: "nowrap", minWidth: "64px" }}>{label}</Body1>
            </div>
            <Body1Strong style={{ width: "100%", overflow: "hidden", textOverflow: "ellipsis" }}>{children}</Body1Strong>
        </div>
    )
}

function PluginDoc({ plugin }: { plugin: PluginManifest }) {
    const { t } = useI18n();
    return (
        <>
            <SettingsGroup title={t('plugin.tabs.details')}>
                <InfoLine icon={<PersonRegular fontSize={20} />} label={t('plugin.details.author')}>
                    {plugin.author}
                </InfoLine>
                {plugin.website && (
                    <InfoLine icon={<GlobeRegular fontSize={20} />} label={t('plugin.details.website')}>
                        <Link onClick={() => openUrl(plugin.website!)} style={{ display: 'flex', alignItems: 'center', justifyContent: "start", gap: '4px', width: "100%" }}>
                            <span style={{ maxWidth: "100%", textWrap: "nowrap", overflow: "hidden", textOverflow: "ellipsis" }}>{plugin.website}</span>
                            <OpenRegular fontSize={16} style={{ flexShrink: 0 }} />
                        </Link>

                    </InfoLine>
                )}
                <InfoLine icon={<InfoRegular fontSize={20} />} label={t('plugin.details.version')}>
                    {plugin.version}
                </InfoLine>
                <InfoLine icon={<NumberSymbolRegular fontSize={20} />} label={t('plugin.details.apiLevel')}>
                    {plugin.api_level}
                </InfoLine>
            </SettingsGroup>

            <SettingsGroup title={t('plugin.details.permissions')}>
                <div className={styles.permissionsContainer}>
                    {plugin.permissions && plugin.permissions.length > 0 ? (
                        plugin.permissions.map(perm => (
                            <div key={perm} className={styles.permissionTag}>
                                <ShieldKeyholeRegular fontSize={14} />
                                <Body1>{perm}</Body1>
                            </div>
                        ))
                    ) : (
                        <Body1>{t('plugin.noFeatures')}</Body1>
                    )}
                </div>
            </SettingsGroup>
        </>
    )
}