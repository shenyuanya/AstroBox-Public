import { useI18n } from "@/i18n";
import { TaskItem } from "@/taskqueue/tasklist";
import {
    Button,
    Dialog,
    DialogBody,
    DialogContent,
    DialogSurface,
    DialogTitle,
    DialogTrigger,
    Input,
} from "@fluentui/react-components";
import { EditRegular } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";

interface ProviderDialogProps {
    item: TaskItem | undefined;
}

export default function ModifyWatchfaceIDDialog({ item }: ProviderDialogProps) {
    const { t } = useI18n();
    const [watchfaceID, setWatchfaceID] = useState<string>("");
    const [disabled, setDisabled] = useState<boolean>(true);

    const modify = async () => {
        setDisabled(true);
        invoke("modify_watchface_id", {
            originalPath: item?.payload?.url,
            newId: watchfaceID
        });
    };
    return (
        <Dialog modalType="alert">
            <DialogTrigger disableButtonEnhancement>
                <Button appearance="subtle" icon={<EditRegular />} size="small">
                </Button>
            </DialogTrigger>
            <DialogSurface style={{ maxWidth: "468px" }}>
                <DialogBody style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
                    <DialogTitle style={{
                        display: "flex",
                        flexDirection: "column",
                        alignItems: "center",
                        gap: "4px"
                    }}>
                        {t('modifyWatchfaceDialog.title')}
                    </DialogTitle>
                    <DialogContent style={{ width: "100%", padding: "0" }}>
                        <Input
                            onChange={(ev, data) => {
                                if (/^\d{9}$|^\d{12}$/.test(data.value)) {
                                    setDisabled(false);
                                    setWatchfaceID(data.value)
                                } else {
                                    setDisabled(true);
                                }
                            }}
                            placeholder={t('modifyWatchfaceDialog.placeholder')}
                            size="large"
                            style={{ width: "100%", marginTop: "18px" }}
                            autoComplete="off"
                            maxLength={12}
                            inputMode="numeric"
                        />
                    </DialogContent>
                    <DialogTrigger>
                        <Button
                            onClick={modify}
                            appearance="primary"
                            size="large"
                            style={{ width: "100%", display: "flex", fontWeight: "500", gap: "8px", marginTop: "20px", fontSize: "15px" }}
                            disabled={disabled}
                        >
                            {t('common.modify')}
                        </Button>
                    </DialogTrigger>
                    <DialogTrigger>
                        <Button appearance="transparent" onClick={() => setDisabled(true)}>{t('common.cancel')}</Button>
                    </DialogTrigger>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    );
}