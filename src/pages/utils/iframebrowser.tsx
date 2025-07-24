import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import BasePage from "@/layout/basePage";
import { Label } from "@fluentui/react-components";
import { useI18n } from "@/i18n";

export default function iframeBrowser() {
    const router = useAnimatedRouter();
    const { t } = useI18n();

    const url = router.query.url as string;
    const tips = router.query.tips as string;

    return (
        <BasePage title={t('iframe.title')}>
            <Label>{tips}</Label>
            <iframe src={url}></iframe>
        </BasePage>
    )
}