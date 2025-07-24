import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import CardButton from "@/components/CardButton/CardButton";
import connect from "@/device/connect";
import { devices } from "@/device/devices";
import { installResourceFromLocal, ResourceType } from "@/device/install";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import { useI18n } from "@/i18n";
import BasePage from "@/layout/basePage";
import useToast, { makeError } from "@/layout/toast";
import { formatBytes } from "@/tools/common";
import { timeToRead } from "@/tools/time";
import { BatteryStatus, ChargeStatus, MiWearState } from "@/types/bluetooth";
import { Button, Caption1, Caption2, InfoLabel, Label, Spinner, Title1 } from "@fluentui/react-components";
import {
    AppsRegular,
    AppsSettingsRegular,
    ArrowSortDownRegular,
    ArrowSortUpRegular,
    ArrowSwapFilled,
    BoxMultipleRegular,
    ClockArrowDownloadRegular,
    ClockToolboxRegular,
    InfoRegular,
    LinkMultipleFilled
} from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import Image from "next/image";
import { useEffect, useState } from "react";
import homeStyles from "../community/community-home/home.module.css";
import styles from "./device.module.css";

export default function Device() {
    const router = useAnimatedRouter();
    const { t } = useI18n();
    const { dispatchToast } = useToast();
    const [deviceState, setDeviceState] = useState<MiWearState | null>();
    const [batteryState, setBatteryState] = useState<BatteryStatus | null>();
    const [loading, setLoading] = useState(false);
    const [currentImageIndex, setCurrentImageIndex] = useState(0);
    const [fadeIn, setFadeIn] = useState(true);

    const updateDeviceBattery = async () => {
        if (!deviceState) return;
        let mass_working = await invoke<boolean>("miwear_is_sending_mass").catch(() => true);
        if (!mass_working) {
            const batteryState = await invoke<BatteryStatus>("miwear_get_device_state").catch(() => null);
            setBatteryState(batteryState);
        }
    }
    const updateDeviceState = async () => {
        let deviceState = await invoke<MiWearState>("miwear_get_state").catch(() => null);
        if (!deviceState) {
            deviceState = await invoke<any>("app_get_config").then((e) => e.current_device).catch(() => null);
            if (!deviceState) return;
            deviceState.disconnected = true;
        }
        setDeviceState(deviceState);
        setCurrentImageIndex(devices.findIndex((e) => e.nameRegex.test(deviceState.name)) ?? 0);
    }
    const reconnect = () => {
        console.log(deviceState);
        if (!deviceState || deviceState.disconnected !== true || !deviceState.authkey) return makeError(dispatchToast, t('device.reconnectFailed'));
        setLoading(true);
        connect(deviceState.addr, deviceState.name, deviceState.authkey).catch((e) => {
            if (e instanceof Error) makeError(dispatchToast, t(e.message));
        }).finally(() => {
            setLoading(false);
        })
    }

    useEffect(() => {
        updateDeviceState();
        const listeners = [
            listen("device-connected", updateDeviceState),
            listen("device-disconnected", updateDeviceState),
        ]
        return () => listeners.forEach(e => e.then(e => e()))
    }, [])
    useEffect(() => {
        if (!deviceState) {
            const interval = setInterval(() => {
                setFadeIn(false);
                setTimeout(() => {
                    setCurrentImageIndex((prev) => (prev + 1) % devices.length);
                    setFadeIn(true);
                }, 500);
            }, 3000);
            return () => clearInterval(interval);
        }
        updateDeviceBattery()
        const getStateInterval = setInterval(updateDeviceBattery, 2000);
        return () => clearInterval(getStateInterval);
    }, [deviceState]);

    return (
        <BasePage title={t('nav.device')} action={<NetworkInfo />}>
            <div className={`${homeStyles.homeComp} ${styles.homeComp}`}>
                <div className={styles.deviceInfo} style={{ width: !deviceState || deviceState?.disconnected ? "100%" : "" }}>
                    {deviceState ? (
                        <>
                            <Image
                                src={devices[currentImageIndex]?.img}
                                alt={devices[currentImageIndex]?.name}
                                width={150}
                                height={150}
                                className={styles.deviceIllustration + " svg"}
                                priority
                            />
                            <div style={{ gap: 5, display: "flex", flexDirection: "column" }}>
                                <div style={{ padding: "0 0 0 14px" }}>
                                    <Caption1 style={{ fontSize: 24, fontWeight: 600, lineHeight: 1.25 }}>{deviceState.name}</Caption1>
                                </div>
                                <div className={styles.deviceStateContainer}>
                                    <Title1 style={{ fontSize: 14, fontWeight: 600, margin: 0, lineHeight: 1.5 }}>{deviceState.disconnected ? t('device.disconnected') : t('device.connected')}</Title1>
                                    {batteryState && <><div className={styles.dividerLine} />
                                        <div className={styles.batteryIcon}>
                                            <div className={styles.batteryIconInner} style={{ width: `${batteryState?.capacity}%`, background: batteryState?.charge_status !== ChargeStatus.Charging ? "var(--colorNeutralForeground1)" : "var(--colorPaletteGreenForeground2)" }} />
                                        </div>
                                        <Caption2 style={{ fontSize: 14, fontWeight: 400 }}>
                                            {batteryState?.capacity}%
                                        </Caption2>
                                        <Caption2 style={{ fontSize: 14, fontWeight: 400 }}>
                                            {batteryState?.charge_status === ChargeStatus.Charging ? t('device.charging') : batteryState?.charge_info?.timestamp && `${t('device.lastChargePrefix')}${timestmpToNow(batteryState?.charge_info?.timestamp)}${t('device.lastChargeSuffix')}`}
                                        </Caption2>
                                    </>}
                                </div>
                                <div style={{ flexDirection: "row", display: "flex" }}>
                                    {(!deviceState || deviceState.disconnected === true) && <AppleButtonWrapper>
                                        <Button appearance="transparent" icon={loading ? <Spinner size="tiny" /> : <LinkMultipleFilled />} onClick={() => reconnect()} style={{ flexShrink: 0 }} disabled={loading}>
                                            {t('device.reconnect')}
                                        </Button>
                                    </AppleButtonWrapper>}
                                    <AppleButtonWrapper>
                                        <Button appearance="transparent" icon={<ArrowSwapFilled />} onClick={() => router.push("/device/switch")} style={{ flexShrink: 0 }}>
                                            {t('device.switch')}
                                        </Button>
                                    </AppleButtonWrapper>
                                </div>

                            </div>
                        </>
                    ) : (
                        <div style={{ textAlign: 'center', display: "flex", alignItems: "center", flexDirection: "row-reverse", justifyContent: "space-between", gap: 16, width: "100%", maxWidth: "364px", padding: 4 }}>
                            <div style={{
                                opacity: fadeIn ? 1 : 0,
                                transition: 'opacity 0.5s'
                            }}>
                                {<Image
                                    src={devices[currentImageIndex].img}
                                    alt={devices[currentImageIndex].name}
                                    width={96}
                                    height={96}
                                    className={styles.deviceIllustration + " svg"}
                                    priority
                                />}
                            </div>
                            <div style={{ display: "flex", alignItems: "start", flexDirection: "column", gap: 8, marginLeft: "-14px", padding: 4 }}>

                                <Label style={{ fontSize: 24, lineHeight: "28px", display: 'block', padding: "0 14px" }}>{t('device.notConnected')}</Label>

                                <AppleButtonWrapper>
                                    <Button appearance="transparent" icon={<ArrowSwapFilled />} onClick={() => router.push("/device/switch")} style={{ flexShrink: 0 }}>
                                        {t('device.connect')}
                                    </Button>
                                </AppleButtonWrapper>
                            </div>
                        </div>
                    )}

                </div>
                <div className={styles.deviceFeatures}
                    style={(!deviceState || deviceState.disconnected === true) ? { display: 'none' } : {}}>
                    <CardButton
                        className={styles.featureBtn}
                        icon={AppsRegular}
                        onClick={() => installResourceFromLocal(ResourceType.ThirdPartyApp)}
                        content={t('device.features.installApp')}
                        secondaryContent={t('device.features.installAppDesc')}
                    />
                    <CardButton
                        className={styles.featureBtn}
                        icon={ClockArrowDownloadRegular}
                        onClick={() => installResourceFromLocal(ResourceType.WatchFace)}
                        content={t('device.features.installWatchface')}
                        secondaryContent={t('device.features.installWatchfaceDesc')}
                        disabled={!deviceState}
                    />
                    <CardButton
                        className={styles.featureBtn}
                        icon={BoxMultipleRegular}
                        onClick={() => installResourceFromLocal(ResourceType.Firmware)}
                        content={t('device.features.installFirmware')}
                        secondaryContent={t('device.features.installFirmwareDesc')}
                        disabled={!deviceState}
                    />
                    <CardButton
                        className={styles.featureBtn}
                        icon={AppsSettingsRegular}
                        content={t('device.features.manageApps')}
                        secondaryContent={t('device.features.manageAppsDesc')}
                        disabled={!deviceState}
                        onClick={() => router.push("/device/app")}
                    />
                    <CardButton
                        className={styles.featureBtn}
                        icon={ClockToolboxRegular}
                        content={t('device.features.manageWatchfaces')}
                        secondaryContent={t('device.features.manageWatchfacesDesc')}
                        disabled={!deviceState}
                        onClick={() => router.push("/device/watchface")}
                    />
                    <CardButton
                        className={styles.featureBtn}
                        icon={InfoRegular}
                        content={t('device.features.deviceInfo')}
                        secondaryContent={t('device.features.deviceInfoDesc')}
                        onClick={() => router.push("/device/info")}
                        disabled={!deviceState}
                    />
                </div>
            </div>
        </BasePage>
    );
}

//某个时间戳到现在的时间
function timestmpToNow(timestamp: number): string {
    const duration = Date.now() - timestamp * 1000;
    return timeToRead(duration);

}

function NetworkInfo() {
    const { t } = useI18n();
    const [network, setNetwork] = useState<{ upload: number, download: number } | null>();
    useEffect(() => {
        const unlisten = listen<{ read: number, write: number }>("network-speed", (e) => {
            const network = { upload: e.payload.read, download: e.payload.write };
            setNetwork(network);
        })
        return () => { unlisten.then(e => e()) };
    }, [])
    if (!network) return null;
    return (
        <InfoLabel
            style={{ display: "flex", alignItems: "center" }}
            info={t('device.networkProxyInfo')}
        >{formatBytes(network.upload)}/s<ArrowSortUpRegular fontSize={16} style={{ marginBottom: -3, padding: "0 4px 0 1px", }} />{formatBytes(network.download)}/s<ArrowSortDownRegular fontSize={16} style={{ marginBottom: -3, padding: "0 4px 0 1px", }} />
        </InfoLabel>
    )

}