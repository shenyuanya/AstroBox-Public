import { useI18n } from "@/i18n";
import { Item } from "@/types/ResManifestV1";
import { Subtitle2 } from "@fluentui/react-components";
import { ArrowRightRegular } from "@fluentui/react-icons";
import React, { useMemo } from "react";
import OnlineImage from "../OnlineImage/OnlineImage";
import styles from "./CommunityWatchfaceCard.module.css";

interface CommunityWatchfaceCardProps {
    item: Item;
    onClick?: (e: React.MouseEvent) => void;
    className?: string;
    preview?: string;
}

const CommunityWatchfaceCardWithPreview: React.FC<CommunityWatchfaceCardProps> = ({
    item,
    preview,
    onClick,
    className,
}) => {
    const { t } = useI18n();
    return (
        <div className={`${styles.card} ${className || ""}`} onClick={onClick} style={{ position: "relative" }}>
            {item.paid_type && (item.paid_type === 'paid' || item.paid_type === 'force_paid') && (
                <span className={styles.paidBadge}>
                    {item.paid_type === 'paid'
                        ? t('resourcePaidType.paid')
                        : t('resourcePaidType.force_paid')}
                </span>
            )}
            <OnlineImage
                src={preview}
                alt=""
                fill
                style={{
                    objectFit: 'cover', borderRadius: "var(--borderRadiusMedium)",
                    overflow: "hidden"
                }} // 或 contain, 按需选择
            />
            <div className={styles.content}>
                <Subtitle2 className={styles.title}>{item.name}</Subtitle2>
                {item.restype && (
                    <div className={styles.restypeContainer}>
                        <div className={styles.restype}>{
                            item.restype === 'quickapp'
                                ? t('resourceType.quickapp')
                                : item.restype === 'watchface'
                                    ? t('resourceType.watchface')
                                    : item.restype === 'firmware'
                                        ? t('resourceType.firmware')
                                        : ''}
                        </div>
                        <ArrowRightRegular style={{ fontSize: 18 }} className={styles.icon} />
                    </div>
                )}
            </div>
        </div>
    );
};
function CommunityWatchfaceCardWithoutPreview({
    item,
    preview,
    onClick,
    className,
}: CommunityWatchfaceCardProps) {
    const { t } = useI18n();
    const nameStyle = useMemo(() => {
        return /^[〈『〖《【〘「〔〚（｛]/.test(item.name || '') ? { textIndent: '-0.6em' } : {}
    }, [item.name])
    return (
        <div className={`${styles.card} ${className || ""}`} onClick={onClick} style={{ position: "relative" }}>
            {item.paid_type && (item.paid_type === 'paid' || item.paid_type === 'force_paid') && (
                <span className={styles.paidBadge}>
                    {item.paid_type === 'paid'
                        ? t('resourcePaidType.paid')
                        : t('resourcePaidType.force_paid')}
                </span>
            )}
            <div className={styles.content2}>
                <div style={{ display: "flex", alignItems: "start", justifyContent: "space-between", width: "100%", height: "100%" }}>
                    <div style={{ height: 48, width: 48, flexShrink: 0 }}>
                        <OnlineImage
                            src={preview ? preview : (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'res.png' : 'res-black.png')}
                            alt=""
                            aspectRatio="1/1"
                            style={{
                                objectFit: 'cover', borderRadius: "999px",
                                overflow: "hidden",
                                margin: "0"
                            }}
                        />
                    </div>

                    <div className={styles.restype} style={{ overflow: "hidden", textOverflow: "ellipsis", whiteSpace: "nowrap" }}>{item._bandbbs_ext_supported_device ? `${item._bandbbs_ext_supported_device}` : ''}
                    </div>
                </div>
                <Subtitle2
                    className={styles.title}
                    style={nameStyle}
                >
                    {item.name}
                </Subtitle2>
                {item.restype && (
                    <div className={styles.restypeContainer}>
                        <div className={styles.restype}>{
                            item.restype === 'quickapp'
                                ? t('resourceType.quickapp')
                                : item.restype === 'watchface'
                                    ? t('resourceType.watchface')
                                    : item.restype === 'firmware'
                                        ? t('resourceType.firmware')
                                        : ''}
                        </div>
                        <ArrowRightRegular style={{ fontSize: 18 }} className={styles.icon} />
                    </div>
                )}
            </div>
        </div>
    )
}
function CommunityWatchfaceCard({ item, ...props }: CommunityWatchfaceCardProps) {
    return item.preview ? <CommunityWatchfaceCardWithPreview {...props} preview={item.preview[0]} item={item} /> : <CommunityWatchfaceCardWithoutPreview {...props} preview={item.icon} item={item} />
}

export default CommunityWatchfaceCard;