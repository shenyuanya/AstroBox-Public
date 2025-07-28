import { useI18n } from "@/i18n";
import {
    Button,
    Dialog,
    DialogActions,
    DialogBody,
    DialogContent,
    DialogSurface,
    DialogTitle,
} from "@fluentui/react-components";
import { useEffect, useRef, useState } from "react";

export interface BoardcastInfo {
    time: string;
    title: string;
    content: string;
    lock_secs: number;
    popup: string;
}

interface BoardcastDialogProps {
    open: boolean;
    onClose: () => void;
    boardcastInfo: BoardcastInfo | null;
}

export default function BoardcastDialog({ open, onClose, boardcastInfo }: BoardcastDialogProps) {
    const { t } = useI18n();
    const [timer, setTimer] = useState(10);
    const intervalRef = useRef<NodeJS.Timeout | null>(null);

    useEffect(() => {
        if (open && boardcastInfo && boardcastInfo.lock_secs > 0) {
            setTimer(boardcastInfo.lock_secs);

            intervalRef.current && clearInterval(intervalRef.current);

            intervalRef.current = setInterval(() => {
                setTimer(prev => {
                    if (prev <= 1) {
                        intervalRef.current && clearInterval(intervalRef.current);
                        return 0;
                    }
                    return prev - 1;
                });
            }, 1000);
        } else {
            setTimer(0);
            intervalRef.current && clearInterval(intervalRef.current);
        }

        return () => {
            intervalRef.current && clearInterval(intervalRef.current);
        };
    }, [open, boardcastInfo]);

    if (!boardcastInfo) return null;

    const handleClose = () => {
        onClose();
    };

    return (
        <Dialog open={open} modalType="alert" onOpenChange={(_, data) => { if (!data.open) handleClose(); }}>
            <DialogSurface>
                <DialogBody>
                    <DialogTitle>{boardcastInfo.title}</DialogTitle>
                    <DialogContent>
                        {boardcastInfo.content.split('\n').map((line, index) => (
                            <p key={index} style={{ margin: '4px 0' }}>{line}</p>
                        ))}
                    </DialogContent>
                    <DialogActions>
                        <Button
                            appearance="primary"
                            onClick={handleClose}
                            disabled={timer > 0}
                        >
                            {`${t('common.ok')}${timer > 0 ? ` (${timer})` : ''}`}
                        </Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    );
}