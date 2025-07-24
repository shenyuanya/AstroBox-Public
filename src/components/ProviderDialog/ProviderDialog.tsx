import {
    Button,
    Dialog,
    DialogActions,
    DialogBody,
    DialogSurface,
    DialogTitle,
    DialogTrigger,
    Spinner,
} from "@fluentui/react-components";
import { Dismiss24Regular } from '@fluentui/react-icons';
import { useEffect, useState } from "react";
import { providerManager } from "@/community/manager";
import { ProviderState } from "@/plugin/types";
import logger from "@/log/logger";
import { useI18n } from "@/i18n";

interface ProviderDialogProps {
    open: boolean;
    onClose: () => void;
}
export default function ProviderDialog({ open, onClose }: ProviderDialogProps) {
    const providers = providerManager.useProviders().providers;
    const [states, setStates] = useState<Record<string, ProviderState>>({});
    const { t } = useI18n();

    useEffect(() => {
        let mounted = true;

        const fetchStates = async () => {
            const entries = await Promise.all(
                providers.map(async (p) => {
                    try {
                        return [p.name, await p.getState()];
                    } catch (err) {
                        logger.error("Get state failed", err);
                        return [p.name, 'Error'];
                    }
                })
            );
            if (mounted) {
                setStates(Object.fromEntries(entries));
            }
        };

        if (providers.length > 0) {
            fetchStates();
            const id = setInterval(fetchStates, 2000);

            return () => {
                mounted = false;
                clearInterval(id);
            };
        }
    }, [providers]);

    const refreshAll = async () => {
        await providerManager.refreshAll();
    };

    return (
        <Dialog open={open} modalType="alert" onOpenChange={(_, d) => !d.open && onClose()}>
            <DialogSurface>
                <DialogBody style={{ display: "flex", flexDirection: "column", rowGap: "8px" }}>
                    <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                        <DialogTitle
                            action={
                                <DialogTrigger action="close">
                                    <Button
                                        appearance="subtle"
                                        aria-label="close"
                                        icon={<Dismiss24Regular />}
                                    />
                                </DialogTrigger>
                            }
                        >
                            {t('providerDialog.title')}
                        </DialogTitle>
                    </div>
                    {providers.length === 0 ? (
                        <Spinner label={t('common.loading')} />
                    ) : (
                        providers.map(prov => (
                            <div key={prov.name} style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
                                <span>{prov.name}</span>
                                <span>{states[prov.name] || t('common.loading')}</span>
                            </div>
                        ))
                    )}
                    <DialogActions>
                        <Button onClick={refreshAll} appearance="primary">{t('common.refreshAll')}</Button>
                    </DialogActions>
                </DialogBody>
            </DialogSurface>
        </Dialog>
    );
}