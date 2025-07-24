import {
    Button,
    Dialog,
    DialogBody,
    DialogContent,
    DialogSurface,
    DialogTitle,
    DialogTrigger,
    Input,
    MenuItem
} from "@fluentui/react-components";
import { ArrowRight16Filled } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { AppInfo } from "@/types/bluetooth";
import { useI18n } from "@/i18n";

interface ProviderDialogProps {
    app: AppInfo | undefined;
}

export default function OpenToPageDialog({app}: ProviderDialogProps) {
    const { t } = useI18n();
    const [page, setPage] = useState<string>("");

    const goto = async () => {
        invoke("miwear_open_quickapp", { app, page: page })
    };
    return (
        <Dialog modalType="alert">
            <DialogTrigger disableButtonEnhancement>
                <MenuItem persistOnClick>{t('openToPageDialog.title')}</MenuItem>
            </DialogTrigger>
                <DialogSurface style={{ maxWidth: "468px" }}>
                    <DialogBody style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
                        <DialogTitle style={{
                            display: "flex",
                            flexDirection: "column",
                            alignItems: "center",
                            gap: "4px"
                        }}>
                            {t('openToPageDialog.title')}
                        </DialogTitle>
                        <DialogContent style={{ width: "100%", padding: "0" }}>
                            <Input
                                onChange={(ev, data) => {
                                    setPage(data.value)
                                }}
                                placeholder={t('openToPageDialog.placeholder')}
                                size="large"
                                style={{ width: "100%", marginTop: "18px" }}
                                autoComplete="off"
                            />
                        </DialogContent>
                        <Button onClick={goto} appearance="primary" size="large" style={{ width: "100%", display: "flex", fontWeight: "500", gap: "8px", marginTop: "20px", fontSize: "15px" }}>{t('common.go')} <ArrowRight16Filled /></Button>
                        <DialogTrigger>
                            <Button appearance="transparent">{t('common.cancel')}</Button>
                        </DialogTrigger>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
    );
}