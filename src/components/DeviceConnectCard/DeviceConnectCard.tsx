import connect from "@/device/connect";
import { devices } from "@/device/devices/devices";
import { useI18n } from "@/i18n";
import useToast, { makeError } from "@/layout/toast";
import logger from "@/log/logger";
import { MiWearState } from "@/types/bluetooth";
import { Body1, Button, Caption1, Card, CardFooter, CardHeader, Field, Input, InputProps, Menu, MenuItem, MenuList, MenuPopover, MenuTrigger, tokens } from "@fluentui/react-components";
import { DeleteRegular, MoreHorizontalRegular, PlugDisconnectedRegular, QrCodeRegular, SendRegular } from "@fluentui/react-icons";
import { Collapse } from '@fluentui/react-motion-components-preview';
import { invoke } from "@tauri-apps/api/core";
import Image from 'next/image';
import { useCallback, useMemo, useState } from "react";
import QRCode from "react-qr-code";
import styles from "./DeviceConnectCard.module.css";

interface DeviceConnectCardProps {
    device: MiWearState;
    connected?: boolean;
    className?: string;
    saved?: boolean;
    onComplete?: () => void;
}

export default function DeviceConnectCard({
    device,
    connected,
    className,
    saved,
    onComplete,
}: DeviceConnectCardProps) {
    const { dispatchToast } = useToast();
    const { t } = useI18n();
    const [showInput, setShowInput] = useState(false);
    const [authKey, setAuthkey] = useState(device.authkey ?? "");
    const [connectStatus, setConnectStatus] = useState(0)//0未链接，1正在链接，3失败
    const handleClick = () => {
        if (connected) return;
        setShowInput(!showInput);
    }

    const onChange: InputProps["onChange"] = (ev, data) => {
        setAuthkey(data.value)
    };

    const connectCb = useCallback(async () => {
        logger.info(authKey, device.addr);
        setConnectStatus(1)
        try {
            await connect(device.addr, device.name, authKey)
            setConnectStatus(2)
            logger.info("Connect success!")
            onComplete?.()
        } catch (error) {
            setConnectStatus(3)
            logger.error(error)
            if (error instanceof Error) {
                const [key, ...rest] = error.message.split(":");
                const detail = rest.join(":");
                makeError(dispatchToast, detail ? `${t(key)}:${detail}` : t(key))
            }
        }
    }, [authKey, device])

    const pic = useMemo(() => devices.find((e) => e.nameRegex.test(device.name))?.miniImg, [device])

    const classNames = [className, styles["device-connect-card"], "card", connected && styles.active].join(" ").trim();
    return (
        <Card className={classNames}
            style={{
                gap: 0,
            }}
            tabIndex={0}
            onKeyUp={(e) => { if (e.key === "Enter") handleClick() }}
        >
            <CardHeader
                role="button"
                onClick={handleClick}
                header={
                    <Body1>
                        <b>{device.name}</b>
                    </Body1>
                }
                description={
                    <Caption1>{device.addr}</Caption1>
                }
                action={
                    <>{saved && <Menu positioning={{ autoSize: true }}>
                        <MenuTrigger disableButtonEnhancement >
                            <Button appearance="subtle" icon={<MoreHorizontalRegular />} onClick={(e) => { e.stopPropagation() }}></Button>
                        </MenuTrigger>

                        <MenuPopover>
                            <MenuList onClick={(e) => { e.stopPropagation() }}>
                                <MenuItem onClick={() => {
                                    invoke("miwear_remove_device",{addr: device.addr}).finally(() => {onComplete?.()})
                                }} icon={<DeleteRegular/>}>{t('device.actions.delete')}</MenuItem>
                                <MenuItem disabled={!connected} onClick={() => {
                                    invoke("miwear_disconnect").finally(() => {onComplete?.()})
                                }} icon={<PlugDisconnectedRegular/>}>{t('device.actions.disconnect')}</MenuItem>
                                <QrMenu device={device} />
                            </MenuList>
                        </MenuPopover>
                    </Menu>}</>
                }
                image={pic ? <Image src={pic} alt={device.name} className="svg" height={32} width={32} /> : undefined}
            ></CardHeader>
            <Collapse visible={showInput}>
                <CardFooter>
                    <Field label={t('device.authkeyPrompt')} required style={{ width: "100%", marginTop: "8px" }}
                        validationMessage={connectStatus == 3 ? t('device.connectFailed') : ""}
                    >
                        <Input onChange={onChange} defaultValue={authKey}
                            contentAfter={
                                <Button appearance="subtle" onClick={() => { connectCb() }}
                                    disabled={connectStatus != 0 && connectStatus != 3}
                                    icon={<SendRegular />}
                                ></Button>
                            }
                            placeholder={t('device.authkeyPlaceholder')}
                            disabled={connectStatus != 0 && connectStatus != 3}
                        />
                    </Field>
                </CardFooter>
            </Collapse>


        </Card>
    )
}
function QrMenu({ device }: { device: MiWearState }) {
    const { t } = useI18n();
    const [qrSize, setQrSize] = useState(128)
    return <Menu persistOnItemClick positioning={{ autoSize: true }}>
        <MenuTrigger disableButtonEnhancement >
            <MenuItem icon={<QrCodeRegular/>}>{t('device.actions.shareQR')}</MenuItem>
        </MenuTrigger>
        <MenuPopover>
            <MenuItem onClick={() => {
                if (qrSize < 256) { setQrSize(qrSize+64) }
                else { setQrSize(64)}
            }}
            subText={t('device.actions.qrResizeHint')}>
                {
                    //@ts-ignore
                    <QRCode title={`${t('device.actions.qrTitle')}${device.name}`} bgColor={tokens.colorNeutralBackground1} fgColor={tokens.colorNeutralForeground1} value={`https://astrobox.online/open?source=deviceQr&name=${device.name}&mac=${device.addr.replaceAll(":", "")}&authkey=${device.authkey}`} size={qrSize} />
                }
            </MenuItem>
        </MenuPopover>
    </Menu>
}