import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import DeviceConnectCard from "@/components/DeviceConnectCard/DeviceConnectCard";
import BasePage from "@/layout/basePage";
import useToast, { makeError } from "@/layout/toast";
import logger from "@/log/logger";
import { BTDeviceInfo, MiWearState } from "@/types/bluetooth";
import { Button, ProgressBar, Skeleton, SkeletonItem } from "@fluentui/react-components";
import { ArrowClockwise20Filled } from "@fluentui/react-icons";
import { checkPermissions } from "@mnlphlp/plugin-blec";
import { Channel, invoke } from "@tauri-apps/api/core";
import Image from 'next/image';
import { useCallback, useEffect, useRef, useState } from "react";
import emptyIllustration from '@/components/illustration/empty.svg';
import noHistoryIllustration from '@/components/illustration/nohistory.svg';
import style from "./switch.module.css";
import { useI18n } from "@/i18n";

export default function Switch() {
    const [state, setState] = useState(false);

    const { t } = useI18n();

    const { dispatchToast } = useToast();
    const [savedDevices, setDevices] = useState<MiWearState[]>([]);

    const getSaved = async () => {
        try {
            const config = await invoke<appconfig>("app_get_config");
            setDevices(config.paired_devices);
        } catch (e) {
            logger.error("get saved error", e);
            makeError(dispatchToast, t('switchDevice.getSavedFailed'));
        }
    }

    useEffect(() => {
        getSaved();
    }, [state]);
    const onComplete = () => {
        setState(!state);
    }

    /* ---------- 通用设备列表渲染 ---------- */
    const DeviceList = useCallback(({
        devices,
        selected,
        saved,
    }: {
        devices: MiWearState[];
        selected?: number;
        saved?: boolean;
    }) => {
        return (
            <div className={style.deviceList}>
                {devices.map((device, index) => (
                    <DeviceConnectCard
                        key={device.addr}
                        device={device}
                        connected={index === selected}
                        saved={saved}
                        onComplete={onComplete}
                    />
                ))}
            </div>
        );
    }, [state])

    /* ---------- 扫描设备列表 ---------- */
    const DeviceScanList = useCallback(({ refreshFlag }: { refreshFlag: boolean }) => {
        const {dispatchToast} = useToast();

        const [devices, setDevices] = useState<BTDeviceInfo[]>([]);
        const channelRef = useRef<Channel<BTDeviceInfo[]> | null>(null);
        const [scanning, setScanning] = useState(false);

        /* 权限检测 */
        const ensurePermission = useCallback(async () => {
            try {
                const ok = await checkPermissions();
                if (!ok) logger.warn("蓝牙权限未授予");
                else logger.info("蓝牙权限正常");
            } catch (e) {
                logger.error("蓝牙设备不存在", e);
            }
        }, []);

        /* 扫描函数 */
        const scan = useCallback(async () => {
            await ensurePermission();

            /* 读取已配对设备地址，用于过滤扫描结果 */
            let savedAddrSet = new Set<string>(savedDevices.map((d) => d.addr));

            const cb = new Channel<BTDeviceInfo[]>();
            cb.onmessage = (newDevices) => {
                setDevices(
                    (newDevices || [])
                        .filter((e) => e.name) // 过滤无名设备
                        .filter((e) => !savedAddrSet.has(e.addr)) // 过滤已配对设备
                );
            };
            channelRef.current = cb;

            setScanning(true);
            try {
                await invoke("miwear_scan", { cb });
                setTimeout(() => setScanning(false), 15000);
            } catch (e) {
                logger.error("扫描出错", e);
                setScanning(false);
                makeError(dispatchToast, t('switchDevice.scanFailed'));
            }
        }, [ensurePermission, dispatchToast]);

        async function startupProcess() {
            const connectStatus = (await invoke<MiWearState>("miwear_get_state").catch(() => null)) ?? null;
            if (!connectStatus) {
                scan();
            }
        }

        /* 组件挂载 & refreshFlag 变化时重新扫描 */
        useEffect(() => {
            startupProcess();
            return () => {
                invoke("miwear_stop_scan");
                logger.info("扫描停止");
                channelRef.current = null;
            };
            // eslint-disable-next-line react-hooks/exhaustive-deps
        }, [refreshFlag]); // refreshFlag 改变时重新扫描

        return (
            <div className={style.listWarpper}>
                <div style={{ display: "flex", flexDirection: "column" }}>
                    <div style={{
                        opacity: scanning ? 1 : 0,
                        transition: 'opacity 0.3s ease-in-out',
                        height: '2px', marginBottom: "-3px"
                    }}>
                        <ProgressBar />
                    </div>
                    <div style={{ display: "flex", flexDirection: "row", alignItems: "center", margin: "0 8px" }}>
                        <h2>{t('switchDevice.scanAndAdd')}</h2>
                        <div style={{ flex: 999 }} />

                        <AppleButtonWrapper padding={5}>
                            <Button
                                disabled={scanning}
                                icon={<ArrowClockwise20Filled style={{
                                    transform: scanning ? 'rotate(360deg)' : 'rotate(0deg)',
                                    transition: 'transform 0.5s ease-in-out'
                                }} />}
                                appearance="transparent"
                                onClick={() => {
                                    if (scanning) return;
                                    scan();
                                }}
                            />
                        </AppleButtonWrapper>

                    </div>
                </div>

                {!devices.length && !scanning && (
                    <div className={style.emptyInformation}>
                        <Image src={emptyIllustration} alt="未找到设备" width="200" height="150" className="svg" />
                        <span style={{ color: "color-mix(in srgb, var(--colorBrandBackgroundInverted) 60%, transparent)" }}>{t('switchDevice.scanNotFound')}</span>
                    </div>
                )}
                {!devices.length && scanning && (
                    <Skeleton aria-label="Scaning Devices" appearance="translucent" style={{
                        display: "flex",
                        flexDirection: "column",
                        alignItems: "center",
                        height: "100%",
                        gap: "5px"
                    }}>
                        <SkeletonItem size={64} style={{ borderRadius: "var(--borderRadiusXLarge)", maxHeight: "60px" }} />
                        <SkeletonItem size={64} style={{ borderRadius: "var(--borderRadiusXLarge)", maxHeight: "60px" }} />
                        <SkeletonItem size={64} style={{ borderRadius: "var(--borderRadiusXLarge)", maxHeight: "60px" }} />
                    </Skeleton>
                )}
                <DeviceList devices={devices as unknown as MiWearState[]} />
            </div>
        );
    }, [state])

    return (
        <>
            <BasePage title={t('switchDevice.title')}>
                <div className={style.context}>
                    <DeviceSavedList />
                    <DeviceScanList refreshFlag={state} />
                </div>
            </BasePage>
        </>
    );

    /* ---------- 已配对设备列表 ---------- */
    function DeviceSavedList() {
        const [selected, setSelected] = useState<number>(-1);

        useEffect(() => {
            (async () => {
                const curDevice =
                    (await invoke<MiWearState>("miwear_get_state").catch(() => null)) ??
                    null;
                setSelected(
                    savedDevices.findIndex((e) => e.addr === curDevice?.addr)
                );
            })()
        }, [state]);

        return (
            <div className={style.listWarpper}>
                <h2 className={style.linkedListTitle}>{t('switchDevice.savedDevices')}</h2>
                {!savedDevices.length &&
                    <div className={style.emptyInformation}>
                        <Image src={noHistoryIllustration} alt="未找到设备" width="200" height="150" className="svg"/>
                        <span style={{ color: "color-mix(in srgb, var(--colorBrandBackgroundInverted) 60%, transparent)" }}>{t('switchDevice.noSavedDevices')}</span>
                    </div>}
                <DeviceList devices={savedDevices} selected={selected} saved />
            </div>
        );
    }

}

interface appconfig {
    current_device: MiWearState;
    paired_devices: MiWearState[];
}