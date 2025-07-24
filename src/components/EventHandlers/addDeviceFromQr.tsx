import { useI18n } from "@/i18n";
import { MiWearState } from "@/types/bluetooth";
import { Button, Dialog, DialogActions, DialogBody, DialogSurface, DialogTitle } from "@fluentui/react-components";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import { useEffect, useMemo, useState } from "react";
import DeviceConnectCard from "../DeviceConnectCard/DeviceConnectCard";

export default function AddDeviceFromQr() {
    const { t } = useI18n();
    const [open, setOpen] = useState(false);
    const [result, setResult] = useState("")
    useEffect(() => {
        const unlisten: Promise<UnlistenFn>[] = []
        unlisten.push(listen<string>("deviceQr", (event) => {
            setResult(event.payload)
            setOpen(true)
        }))
        return () => {
            unlisten.forEach(e => e.then(e => e()))
        }
    })
    const device = useMemo(() => {
        if (!result) return {
            name: "",
            addr: "",
            connect_type: "SPP"
        } satisfies MiWearState
        const url = new URL(result)
        const name = url.searchParams.get("name")
        let addr = url.searchParams.get("mac")
        const authkey = url.searchParams.get("authkey") ?? ""
        if (!name || !addr) {
            setResult("")
            setOpen(false)
            return {
                name: "",
                addr: "",
                connect_type: "SPP"
            } satisfies MiWearState
        }
        addr = addr.split(/(.{2})/u).filter(Boolean).join(":")
        return {
            name,
            addr,
            authkey,
            connect_type: "SPP"
        } satisfies MiWearState
    }, [result])
    return (<Dialog open={open} modalType="alert">
        <DialogSurface>
            <DialogBody style={{display:"flex",flexDirection:"column"}}>
                <DialogTitle>{t('addDeviceFromQr.title')}</DialogTitle>
                {device.addr&&<DeviceConnectCard device={device} onComplete={()=>setOpen(false)}></DeviceConnectCard>}
            </DialogBody>
            <DialogActions>
                <Button onClick={()=>setOpen(false)}>{t('common.cancel')}</Button>
            </DialogActions>
        </DialogSurface>
    </Dialog>)
}