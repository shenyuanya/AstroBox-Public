import BasePage from "@/layout/basePage";
import SettingsGroup from "@/layout/settingsGroup";
import {BatteryStatus, MiWearState, SystemInfo} from "@/types/bluetooth";
import {Body1Strong, Body2, Caption1, tokens} from "@fluentui/react-components";
import {AnimatePresence} from "framer-motion";
import {useState} from "react";
import useInvoke, {useInvokeWithMass, useIsSendingMass} from "@/hooks/useInvoke";
import { useI18n } from "@/i18n";

export default function Info() {
    const {data: deviceInfo} = useInvoke<MiWearState>()("miwear_get_state")
    const {isSendingMass} = useIsSendingMass();
    const {data: deviceInfo2} = useInvokeWithMass<SystemInfo>(isSendingMass)("miwear_get_device_info")
    const {data: batteryState} = useInvokeWithMass<BatteryStatus>(isSendingMass)("miwear_get_device_state")
    const { t } = useI18n();
    return <BasePage title={t('deviceInfo.title')}>
        <AnimatePresence>
            <Caption1 style={{ margin: "0 8px", color: tokens.colorStatusWarningForeground3 }}>{t('deviceInfo.notRealtime')}</Caption1>
            {deviceInfo && <SettingsGroup title={t('deviceInfo.groups.device')}>
                <InfoCard name={t('deviceInfo.fields.name')} value={deviceInfo?.name} />
                <InfoCard name={t('deviceInfo.fields.address')} value={deviceInfo?.addr} />
                <InfoCard name={t('deviceInfo.fields.authkey')} value={deviceInfo?.authkey} serect />
                <InfoCard name={t('deviceInfo.fields.connectionType')} value={deviceInfo?.connect_type} />
                <InfoCard name={t('deviceInfo.fields.codename')} value={deviceInfo?.codename} />
            </SettingsGroup>}
            {deviceInfo2 && <SettingsGroup title={t('deviceInfo.groups.system')}>
                <InfoCard name={t('deviceInfo.fields.model')} value={deviceInfo2?.model} />
                <InfoCard name={t('deviceInfo.fields.imei')} value={deviceInfo2?.imei} serect />
                <InfoCard name={t('deviceInfo.fields.firmware')} value={deviceInfo2?.firmware_version} />
                <InfoCard name={t('deviceInfo.fields.serial')} value={deviceInfo2?.serial_number} serect />
            </SettingsGroup>}
            {batteryState && <SettingsGroup title={t('deviceInfo.groups.status')}>
                <InfoCard name={t('deviceInfo.fields.battery')} value={batteryState?.capacity.toFixed(0) + "%"} />
                <InfoCard name={t('deviceInfo.fields.chargeStatus')} value={batteryState?.charge_status.toString()} />
                {batteryState?.charge_info?.timestamp && <InfoCard name={t('deviceInfo.fields.lastCharge')} value={new Date(batteryState?.charge_info?.timestamp * 1000).toLocaleString()} />}
            </SettingsGroup>}
        </AnimatePresence>

    </BasePage>
}
function InfoCard({ name, value, serect }: { name: string, value?: string, serect?: boolean }) {
    const [showSecret, setShowSecret] = useState(!serect);
    if (!value) return null;
    return <div className="card" style={{ justifyContent: "space-between", display: "flex" }} onClick={() => { setShowSecret(!showSecret || !serect) }}>
        <Body1Strong className="name">{name}</Body1Strong>
        <Body2 className="value" onClick={() => {
            navigator.clipboard.writeText(value).then(() => {
                console.log("Text copied to clipboard");
            });
        }}>{showSecret ? value : Array(value.length).fill("*").join("")}</Body2>
    </div>
}