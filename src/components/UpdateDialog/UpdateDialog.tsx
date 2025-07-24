import { useI18n } from "@/i18n";
import {
    Button,
    Checkbox,
    Dialog,
    DialogActions,
    DialogBody,
    DialogContent,
    DialogSurface,
    DialogTitle,
} from "@fluentui/react-components";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useState } from "react";

export interface UpdateInfo {
    version: string;
    url: string;
    time: string;
    changelog: string;
}

interface UpdateDialogProps {
    open: boolean;
    onClose: () => void;
    updateInfo: UpdateInfo | null;
    onIgnore: (version: string) => void;
}

export default function UpdateDialog({ open, onClose, updateInfo, onIgnore }: UpdateDialogProps) {
    const { t } = useI18n();
    const [dontRemind, setDontRemind] = useState(false);

    if (!updateInfo) {
        return null;
    }

    const handleClose = () => {
        if (dontRemind) {
            onIgnore(updateInfo.version);
        }
        onClose();
    };

    const handleUpdateNow = () => {
        openUrl(updateInfo.url);
        handleClose();
    };

    return (
        <Dialog open={open} onOpenChange={(_, data) => { if (!data.open) handleClose(); }}>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>{t('updateDialog.title')?.replace('{version}', updateInfo.version)}</DialogTitle>
                    <DialogContent>
                        <p>{t('updateDialog.body')}</p>
                        {updateInfo.changelog.split('\n').map((line, index) => (
                            <p key={index} style={{ margin: '4px 0' }}>{line}</p>
                        ))}
                    </DialogContent>
                    <Checkbox
                        label={t('updateDialog.ignore')}
                        style={{
                            marginLeft: "-7px"
                        }}
                        checked={dontRemind}
                        onChange={(_, data) => setDontRemind(!!data.checked)}
                    />
                    <DialogActions>
                        <Button appearance="secondary" onClick={handleClose}>
                            {t('updateDialog.later')}
                        </Button>
                        <Button appearance="primary" onClick={handleUpdateNow}>
                            {t('updateDialog.updateNow')}
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    );
}