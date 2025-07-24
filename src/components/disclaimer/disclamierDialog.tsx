import { useI18n } from "@/i18n";
import { Body1, Button, Dialog, DialogActions, DialogBody, DialogContent, DialogSurface, DialogTitle, DialogTrigger } from "@fluentui/react-components";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

export function DisclaimerDialog({defaultOpen = false,trigger}:{defaultOpen?:boolean,trigger?:React.ReactElement}) {
    const { t } = useI18n();
    const [canClose, setCanClose] = useState(false);

    return <Dialog defaultOpen={defaultOpen} modalType="alert">
        <DialogTrigger>
            {trigger}
        </DialogTrigger>
        <DialogSurface>
            <DialogBody>
<DialogTitle>{t('disclaimer.title')}</DialogTitle>
            <DialogContent onScroll={(e) => {
                setCanClose((e.target as HTMLElement).scrollTop > (e.target as HTMLElement).clientHeight*2);
            }}>
                <Body1>{t('disclaimer.content').split('\n').map((item,index)=>{
                    return <>
                        <p key={index}>{item}</p>
                    </>
                })}</Body1>
            </DialogContent>
            <DialogActions>
                <DialogTrigger>
                    <Button appearance="primary" disabled={!canClose} onClick={()=>{localStorage.setItem("disclaimerAccepted","true");}}>{t('disclaimer.confirm')}</Button>
                </DialogTrigger>
                <Button onClick={() => { invoke("cleanup_before_exit") }}>{t('disclaimer.cancel')}</Button>
            </DialogActions>
            </DialogBody>

        </DialogSurface>
    </Dialog>
}
