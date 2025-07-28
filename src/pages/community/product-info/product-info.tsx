import {providerManager} from "@/community/manager";
import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import {useAnimatedRouter} from "@/hooks/useAnimatedRouter";
import useDeviceMap from "@/hooks/useDeviceMap";
import useIsMobile from "@/hooks/useIsMobile";
import {useI18n} from "@/i18n";
import BasePage from "@/layout/basePage";
import logger from "@/log/logger";
import {Provider} from "@/plugin/types";
import {createDownloadTask} from "@/taskqueue/downloadTask";
import {addDownloadTask, useDownloadQueue, useInstallQueue} from "@/taskqueue/queue";
import {MiWearState} from "@/types/bluetooth";
import {ResourceManifestV1} from "@/types/ResManifestV1";
import {
    Button,
    Carousel,
    CarouselCard,
    CarouselNavContainer,
    CarouselSlider,
    CarouselViewport,
    makeStyles,
    Menu,
    MenuButtonProps,
    MenuItem,
    MenuList,
    MenuPopover,
    MenuTrigger,
    Spinner,
    Subtitle1,
    Subtitle2,
    tokens
} from "@fluentui/react-components";
import {
    AppsRegular,
    CheckmarkCircleFilled,
    ChevronDownFilled,
    ClipboardRegular,
    DismissCircleFilled,
    LinkMultipleFilled,
    ShareIosFilled
} from "@fluentui/react-icons";
import {invoke} from "@tauri-apps/api/core";
import {openUrl} from "@tauri-apps/plugin-opener";
import ColorThief from "color-thief-browser";
import parse from 'html-react-parser';
import Image from "next/image";
import {useEffect, useMemo, useRef, useState} from "react";
import QRCode from "react-qr-code";
import AutoSizer from "react-virtualized-auto-sizer";
import styles from "./productinfo.module.css";

const useClasses = makeStyles({
    btn: {
        backdropFilter: "blur(30px)",
        borderRadius: "999px",
        background: tokens.colorNeutralBackgroundAlpha,
    },
})

export default function ProductInfo() {
    const classes = useClasses();
    const { t } = useI18n();
    const [loading, setLoading] = useState(true);
    const [errInfo, setErrInfo] = useState("");
    const [curItem, setCurItem] = useState<ResourceManifestV1>();
    // 图标取色
    const [dominantColor, setDominantColor] = useState<[number, number, number] | null>(null);
    const [fgColor, setFgColor] = useState<string | null>(null);
    const [device, setDevice] = useState<MiWearState | null>(null);

    const [imgsrc, setImg] = useState<any>(null);
    const imgRef = useRef<HTMLImageElement>(null);
    const router = useAnimatedRouter();
    const { providers } = providerManager.useProviders();

    const providerName = router.query.provider as string;
    const provider = providerManager.get(providerName);
    const name = router.query.name as string;

    const pickColor = () => {
        const img = imgRef.current;
        if (!img || imgsrc !== img.src) return;
        try {
            const colorThief = new ColorThief();
            const color = colorThief.getColor(img);
            setDominantColor(color);

            const fgResult = getContrastSafeCssColor(color, {
                fg: color,
                bg: color,
                threshold: 4.5,
                fallback: "fallback", // 特殊标识用作判定
                prefer: [
                    { name: 'var(--colorNeutralForegroundStaticInverted)', rgb: [255, 255, 255] },
                    { name: 'var(--colorNeutralForeground1Static)', rgb: [0, 0, 0] }
                ]
            });

            setFgColor(fgResult.color);
        } catch (error) {
            console.warn("提取颜色失败：", error);
        }
    };

    const loadItem = async () => {
        try {
            if (!provider) return;
            const ret = await provider.getItem(name);
            setCurItem(ret);
            setLoading(false);
            invoke<string>("image_url_to_base64_data_url", {
                url: ret?.item.icon
            }).then((data) => {
                setImg(data);
            })
            const config = await invoke<any>("app_get_config").catch(_ => null);
            const device = config?.current_device ? { ...config.current_device } : null;
            const codename = await invoke<string>("miwear_get_codename").catch(_ => null);
            if (device) {
                device.codename = codename ?? device.codename;
                setDevice(device);
            } else {
                setDevice(null);
            }

        } catch (e) {
            //@ts-ignore
            logger.warn(e.toString());
            // 报错就是资源不存在 不做任何处理
            setErrInfo(t('product.notFound'));
        }
    };

    useEffect(() => {
        loadItem();
    }, [providers]);
    return (
        <BasePage
            title={t('product.detail')}
            {...(dominantColor && { arrowColor: `rgba(${dominantColor.join(",")}, 1)` })}
        // action={<SearchBox placeholder="搜索表盘、快应用..." className={styles["search-box"]} />}
        >
            <div className={styles.linearBg} style={{
                background: dominantColor ? `rgba(${dominantColor.join(",")}, 1)` : "transparent",
                transition: "background 0.25s ease-in"
            }} >
            </div>
            {loading &&
                <div className={styles.loadingBox}>
                    {!errInfo && <Spinner
                    // label="加载中" labelPosition="below"
                    ></Spinner>}
                    {errInfo && <div style={{ color: "gray" }}>{errInfo}</div>}
                </div>
            }
            {!loading &&
                <div style={{
                    ["--bg-color" as string]: dominantColor ? `rgba(${dominantColor.join(",")}, 1)` : "var(--colorBrandBackground)",
                    ["--fg-color" as string]: fgColor ? fgColor : "var(--colorNeutralForeground1Static)",
                    zIndex: 1
                }} >
                    {/* 标题 */}
                    <div className={styles.productInfoContainer}>
                        <div className={styles.productTitle}>
                            <Image ref={imgRef} onLoad={pickColor}
                                src={imgsrc ? imgsrc : (window.matchMedia('(prefers-color-scheme: dark)').matches ? 'res.png' : 'res-black.png')}
                                alt="productIcon" height={52} width={52} crossOrigin=""
                                className={imgsrc ? "" : "svg"} style={{ borderRadius: "999px" }} />
                            <div className={styles.productInfo} style={{ gap: 0 }}>
                                <label className={styles.productInfoTitle}>
                                    {curItem?.item?.name}
                                </label>
                                <div className={styles.productInfoAuthors}>
                                    {Array.isArray(curItem?.item?.author) && curItem.item.author.map((author, idx) =>
                                        author.author_url && author.author_url.trim() ? (
                                            <a
                                                key={idx}
                                                href={author.author_url}
                                                target="_blank"
                                                rel="noopener noreferrer"
                                                className={styles.productInfoLink}
                                                style={{ marginRight: 10 }}
                                            >
                                                @{author.name}
                                            </a>
                                        ) : (
                                            <span key={idx} className={styles.productInfoLink} style={{ marginRight: 8 }}>
                                                @{author.name}
                                            </span>
                                        )
                                    )}
                                </div>
                            </div>
                        </div>
                        {/* <div className={styles.productTagsContainer}>
                            <div className={styles.productTags}>
                                <p>v1.0.0</p>
                                <span>版本</span>
                            </div>
                        </div> */}
                    </div>
                    <div className={styles.productInfoContainer}>
                        <div className={styles.productInfo}>
                            {Array.isArray(curItem?.item?.preview) && <p>{curItem?.item?.description}</p>}
                            <div className={styles.btnGroup}>
                                {curItem && <DownloadBtn manifest={curItem} codename={device?.codename} />}
                                <div className={styles.shareBtnGroup}>
                                    {curItem && provider && <ShareButton
                                        manifest={curItem!}
                                        provider={provider!}
                                    ></ShareButton>}
                                    {curItem?.item._bandbbs_ext_resource_id && <AppleButtonWrapper >
                                        <Button icon={<LinkMultipleFilled />} appearance="transparent" onClick={() => {
                                            openUrl(`https://www.bandbbs.cn/resources/${curItem?.item._bandbbs_ext_resource_id}`)
                                        }}>
                                            {t('productInfo.viewOnBandBBS')}
                                        </Button>
                                    </AppleButtonWrapper>}

                                </div>
                            </div>
                        </div>
                    </div>
                    {/* 屏幕截图 */}
                    {Array.isArray(curItem?.item?.preview) && <AutoSizer disableHeight className={styles.previewScroller}>
                        {({ width }) => (
                            <Carousel draggable whitespace align="center" autoplayInterval={5000} circular className={styles.carousel}
                                style={{ overflow: "visible", width }}
                            >
                                <CarouselViewport>
                                    <CarouselSlider>
                                        {Array.isArray(curItem?.item?.preview) && curItem.item.preview.map((url, idx) => (
                                            <CarouselCard key={url} autoSize style={{ padding: "0 5px" }}>
                                                <img
                                                    src={url}
                                                    alt={`screenshot-${idx + 1}`}
                                                    className={styles.previewImage}
                                                />
                                            </CarouselCard>
                                        ))}
                                    </CarouselSlider>
                                </CarouselViewport>
                                <CarouselNavContainer
                                    layout="overlay-expanded"
                                    next={{
                                        className: classes.btn,
                                    }}
                                    prev={{
                                        className: classes.btn,
                                    }}
                                >
                                </CarouselNavContainer>
                            </Carousel>)}
                    </AutoSizer>}
                    {/* 资源信息 */}
                    {!Array.isArray(curItem?.item?.preview) &&
                        <div className={styles.productInfoContainer} ref={(ref) => { ref?.querySelectorAll("a")?.forEach((e) => { e.target = "_blank" }) }}>
                            <div className={styles.productInfo}>
                                <Subtitle1>{t('productInfo.about')}</Subtitle1>
                                <br />
                                {curItem?.item.description && parse(curItem?.item.description)}
                            </div>
                        </div>}
                    {/* 版本更新 */}
                    {/*
                        <div className={styles.productInfoContainer}>
                            <div className={styles.productInfo}>
                                <Subtitle1 style={{ display: "flex", alignItems: "center" }}>更新<IosChevronRightFilled style={{ fontSize: 20 }} /></Subtitle1>

                                <Subtitle2>v1.0.0 更新</Subtitle2>
                                <p>优化表盘的稳定性，增强表盘的流畅度。</p>
                                <p>2025.7.5</p>
                            </div>
                        </div>
                        */}
                    {/* 系统要求 */}
                    {device && device.codename && !curItem?.item._bandbbs_ext_resource_id && <div className={styles.productInfoContainer}>
                        <div className={styles.productInfo}>
                            <Subtitle1>{t('productInfo.deviceRequirements')}</Subtitle1>
                            {curItem?.downloads.hasOwnProperty(device?.codename ?? "fuckyouanyway") ?
                                <Subtitle2 style={{ display: "flex", alignItems: "flex-start", gap: 4, padding: "8px 0" }}><CheckmarkCircleFilled style={{ fontSize: 20, color: tokens.colorStatusSuccessForeground3, flexShrink: 0, marginTop: "1px" }} />{t('productInfo.compatible')} {device?.name}</Subtitle2>



                                : <Subtitle2 style={{ display: "flex", alignItems: "flex-start", gap: 4, padding: "8px 0" }}><DismissCircleFilled style={{ fontSize: 20, color: tokens.colorStatusDangerForeground3, flexShrink: 0, marginTop: "1px" }} />{t('productInfo.incompatible')} {device?.name} {t('productInfo.incompatibleSuffix')}</Subtitle2>
                            }
                            <p>{t('productInfo.otherVersions')}</p>
                        </div>
                    </div>}
                </div>
            }
        </BasePage >
    )
}

function DownloadBtn({ manifest: { downloads, item: { icon, name, description, _bandbbs_ext_resource_id } }, codename }: { manifest: ResourceManifestV1, codename?: string | null }) {
    const router = useAnimatedRouter()
    const { t } = useI18n();
    const deviceMap = useDeviceMap();
    const isMobile = useIsMobile()
    const { items: downloadItems } = useDownloadQueue();
    const { items: installItems } = useInstallQueue();
    const list = []
    for (const key of Object.keys(downloads)) {
        const item = downloads[key];
        list.push({ key, item })
    }

    let hasItem = useMemo(() => {
        const items = [...downloadItems, ...installItems];
        //@ts-ignore
        return items.findIndex(item => item.id === (downloads[codename]?.file_name ?? name ?? "")) != -1;
    }, [downloadItems, installItems, codename]);

    if (list.length === 0) return null;
    const providerName = router.query.provider as string;
    description = _bandbbs_ext_resource_id ? "" : description;
    const download = async (code: string) => {
        const iconComponent = () => icon ? <Image width={40} height={40} src={icon} alt={name ?? ""} style={{borderRadius:999}}/> : <AppsRegular />;
        const taskId = _bandbbs_ext_resource_id?.toString() ?? name ?? "";
        const taskName = name ?? "";
        const displayDescription = (code === codename ? "" : `(${code}) `) + (description ?? "");
        const task = createDownloadTask(
            taskId,
            taskName,
            providerName,
            code,
            displayDescription,
            iconComponent
        );
        addDownloadTask(task);
    }
    const showDefault = codename && downloads.hasOwnProperty(codename)
    return (
        <Menu positioning="below-end">
            <MenuTrigger disableButtonEnhancement>
                {(triggerProps: MenuButtonProps) => (
                    <AppleButtonWrapper padding={5}>
                        <div className={styles.downloadBtn} style={{ width: isMobile ? "100%" : undefined }}>
                            {showDefault && (<><Button onClick={() => download(codename)} style={{ flex: 1, borderRadius: 0, color: "var(--fg-color)" }} appearance="transparent" disabled={hasItem}>
                                {hasItem ? t('productInfo.inQueue') : t('common.download')}
                            </Button>
                                <div style={{ width: "1px", height: "100%", background: "var(--fg-color)", opacity: 0.2 }} /></>)}
                            <Button {...triggerProps} iconPosition="after" appearance="transparent" style={{ borderRadius: 0, color: "var(--fg-color)", width: "100%", justifyContent: "space-between" }} icon={<ChevronDownFilled style={{ flexShrink: 0 }} />}>
                                {!showDefault && t('common.download')}
                            </Button>
                        </div>
                    </AppleButtonWrapper>
                )}
            </MenuTrigger>

            <MenuPopover>
                <MenuList>
                    {list.map((item) => (
                        <MenuItem key={item.key} onClick={() => download(item.key)}>
                            {
                                Object.values(deviceMap || {}).find(device => device.codename === item.key)?.name
                                ?? item.key
                            }
                        </MenuItem>
                    ))}
                </MenuList>
            </MenuPopover>
        </Menu>
    );
}
function ShareButton({ manifest: { item }, provider }: { manifest: ResourceManifestV1, provider: Provider }) {
    const { t } = useI18n();
    const link = encodeURI(`https://astrobox.online/open?source=res&res=${item._bandbbs_ext_resource_id?item._bandbbs_ext_resource_id:item.name}&provider=${provider.name}`)
    const [qrSize, setQrSize] = useState(128)
    return (
        <Menu>
            <MenuTrigger disableButtonEnhancement>
                {(triggerProps) =>
                    <AppleButtonWrapper padding={5}>
                        <Button {...triggerProps} icon={<ShareIosFilled />} appearance="transparent">
                            {t('common.share')}
                        </Button>
                    </AppleButtonWrapper>
                }
            </MenuTrigger>
            <MenuPopover>
                <MenuList>
                    <MenuItem icon={<ClipboardRegular />} onClick={() => {
                        navigator.clipboard.writeText(link);
                    }}>{t('productInfo.copyLink')}</MenuItem>
                    <MenuItem onClick={() => {
                        if (qrSize < 256) { setQrSize(qrSize + 64) }
                        else { setQrSize(64) }
                    }}
                        subText={t('productInfo.resizeQr')}>
                        {
                            //@ts-ignore
                            <QRCode bgColor={tokens.colorNeutralBackground1} fgColor={tokens.colorNeutralForeground1} value={link} size={qrSize} />
                        }
                    </MenuItem>
                </MenuList>
            </MenuPopover>
        </Menu>

    )
}

type RGBTuple = [number, number, number];

function sRGBtoLinear(c: number): number {
    c /= 255;
    return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
}

function getLuminance(r: number, g: number, b: number): number {
    return 0.2126 * sRGBtoLinear(r) + 0.7152 * sRGBtoLinear(g) + 0.0722 * sRGBtoLinear(b);
}

function contrastRatio(l1: number, l2: number): number {
    return (Math.max(l1, l2) + 0.05) / (Math.min(l1, l2) + 0.05);
}

/**
 * 返回与背景有足够对比度的颜色字符串（CSS rgba / CSS 变量），否则使用 fallback。
 */
export function getContrastSafeCssColor(
    color: [number, number, number],
    {
        fg,
        bg,
        threshold = 4.5,
        fallback = 'var(--colorBrandForeground1)',
        prefer = [
            { name: 'black', rgb: [0, 0, 0] },
            { name: 'white', rgb: [255, 255, 255] },
        ],
    }: {
        fg: RGBTuple;
        bg: RGBTuple;
        threshold?: number;
        fallback?: string;
        prefer?: { name: string; rgb: RGBTuple; }[];
    }
): { color: string, isFallback: boolean } {
    const fgL = getLuminance(...fg);
    const bgL = getLuminance(...bg);
    const baseContrast = contrastRatio(fgL, bgL);

    if (baseContrast >= threshold) {
        return { color: `rgba(${fg.join(',')}, 1)`, isFallback: false };
    }

    for (const candidate of prefer) {
        const candidateL = getLuminance(...candidate.rgb);
        const contrast = contrastRatio(candidateL, bgL);
        if (contrast >= threshold) {
            return { color: candidate.name, isFallback: true };
        }
    }

    return { color: fallback, isFallback: true };
}