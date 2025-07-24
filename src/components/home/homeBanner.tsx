import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import useIsMobile from "@/hooks/useIsMobile";
import { useI18n } from "@/i18n";
import { BannerItem } from "@/types/BannerItem";
import {
    Button,
    Carousel,
    CarouselCard,
    CarouselNav,
    CarouselNavButton,
    CarouselNavContainer,
    CarouselSlider,
    CarouselViewport,
    makeStyles,
    Skeleton,
    SkeletonItem,
    Title1,
    tokens,
    typographyStyles
} from "@fluentui/react-components";
import { ChevronRightFilled, ImageSplitRegular } from "@fluentui/react-icons";
import ColorThief from "color-thief-browser";
import Image from "next/image";
import { Suspense, useRef, useState } from "react";
import classes from "./communityhome.module.css";

import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import useInvoke from "@/hooks/useInvoke";
import { openUrl } from "@tauri-apps/plugin-opener";

const useStyle = makeStyles(
    {
        card: {
            height: 'fit-content',
            aspectRatio: '541 / 249',
            backgroundSize: 'cover',
            maxWidth: '748px',
            position: "relative",
            margin: "0 10px",
            borderRadius: tokens.borderRadiusLarge,
            pointerEvents: "none",
            flexDirection: "row",
            display: "flex",
            boxShadow: "0 0 5px 0 rgba(0,0,0,0.5)",
            transition: "boxShadow 0.3s ease-in-out",
        },
        root: {
            width: "calc(100vw - 32px)",
            "@media (min-width: 768px)": {
                width: "80vw",
            },
            maxWidth: '748px',
            margin: "12px auto",
            borderRadius: tokens.borderRadiusLarge,
            overflow: "visable",
        },
        btn: {
            backdropFilter: "blur(30px)",
            borderRadius: "999px",
            margin: "0 -30px",
            background: tokens.colorNeutralBackgroundAlpha,
        },
        title: {
            ...typographyStyles.title2,
            padding: "0 20px",
            margin: "0",
            marginTop: "20px"
        },
        desc: {
            ...typographyStyles.body1,
            padding: "0 20px",
            margin: "0 0 10px 0",
            width: "80%",
            textOverflow: "ellipsis",
            overflow: "hidden",
        },
        cardInfo: {
            display: "flex",
            flexDirection: "column",
            justifyContent: "center",
            alignItems: "left",
            pointerEvents: "all",
            zIndex: 1,
            gap: 0
        },
        fgImg: {
            zIndex: 1,
            flex: 1,
            width: "70%",
            height: "100%",
            objectFit: "contain",
            margin: "auto 0",
            pointerEvents: "none"
        }
    }
)

function BannerCard({item}: { item: BannerItem }) {
    const [dominantColor, setDominantColor] = useState<[number, number, number] | null>(null);
    const imgRef = useRef<HTMLImageElement>(null);
    const isMobile = useIsMobile();
    const router = useAnimatedRouter();

    const handleCardClick = () => {
        if (item.button.url.startsWith("https://astrobox.online")) {
            const url = new URL(item.button.url);
            router.push({
                pathname: '/community/product-info/product-info',
                query: { name: decodeURIComponent(url.searchParams.get("res")??""), provider: url.searchParams.get("provider") },
            });
        } else {
            openUrl(item.button.url);
        }
    };

    const handleLoad = async () => {
        try {
            const img = imgRef.current;
            // 创建新的ColorThief实例
            const colorThief = new ColorThief();
            const color = colorThief.getColor(img!);
            setDominantColor(color);
        } catch (error) {
            console.warn("提取颜色失败：", error);
        }
    };
    const textColor = !dominantColor ? "#000" : (() => {
        const [r, g, b] = dominantColor.map(c => {
            c = c / 255;
            // 使用更精确的sRGB转换
            return c <= 0.03928 ? c / 12.92 : Math.pow((c + 0.055) / 1.055, 2.4);
        });
        const luminance = 0.2126 * r + 0.7152 * g + 0.0722 * b;
        // 调整阈值到0.5，这样更符合WCAG 2.0的对比度要求
        return luminance > 0.5 ? "#000" : "#fff";
    })();
    const styles = useStyle();
    return (
        <CarouselCard className={styles.card} as="div" onClick={handleCardClick}>
            {/* {selected && <Image src={item.background} alt={item.title!} fill style={{ filter: "brightness(0.6) blur(20px)", zIndex: -1, transition: "filter 0.3s ease-in-out" }} />} */}
            <Image ref={imgRef} src={item.background} alt={item.title!} onLoad={() => handleLoad()} fill
                   style={{borderRadius: tokens.borderRadiusLarge, objectFit: "cover"}} crossOrigin="anonymous"/>
            <div style={{
                position: "absolute",
                width: "100%",
                height: "100%",
                borderRadius: tokens.borderRadiusLarge,
                background: `linear-gradient(to right, rgba(${dominantColor?.join(",") ?? "255,255,255"}, 0.9) 25%, rgba(${dominantColor?.join(",") ?? "255,255,255"}, 0.25) 50%)`,
            }}></div>
            <div className={styles.cardInfo}>
                <div style={{ flex: 2 }}></div>
                <Title1 style={{ color: textColor }} className={styles.title}>{item.title}</Title1>
                <p className={styles.desc} style={{ color: textColor }}>{item.description!}</p>
                {!isMobile && item.button.text && <div style={{margin: "0 0 20px 20px "}}>
                    <AppleButtonWrapper transition="box-shadow 0.2s ease-in-out">
                        <Button appearance="outline" className={classes.cardButtons}
                                icon={<ChevronRightFilled/>}>{item.button.text}</Button>
                    </AppleButtonWrapper>
                </div>}
            </div>

            {item.foreground ? //@ts-ignore
                <img src={item.foreground.src ?? item.foreground} alt={item.title!} className={styles.fgImg}/> :
                <ImageSplitRegular className={styles.fgImg}/>}
        </CarouselCard>
    )
}

export default function Banner() {
    const { t } = useI18n();
    const styles = useStyle();
    const ismobile = useIsMobile();

    let { data: items, isLoading, error } = useInvoke<BannerItem[]>({
        keepPreviousData: true,
    })("officialprov_get_banners")
    const [activeIndex, setIndex] = useState(0);

    if (isLoading) return <Skeleton aria-label="Loading Content" className={styles.root}
                                    style={{display: "flex", flexDirection: "row", gap: 10}} appearance="translucent">
        {Array(2).fill(1).map((_, i) => <SkeletonItem key={i} className={styles.card}
                                                      style={{flexShrink: "0", margin: 0}}/>)}
    </Skeleton>
    if (error) items = [{
        title: t('banner.loadFailed.title'),
        description: error.toString(),
        background: "",
        foreground: "",
        button: {
            url: ""
        }
    }]
    if (!items) items = [{
        title: t('banner.noBanners.title'),
        description: t('banner.noBanners.description'),
        background: "",
        foreground: "",
        button: {
            url: ""
        }
    }]

    return (
        <Suspense fallback={<p>{t('banner.loading')}</p>}>
        <Carousel groupSize={1} whitespace draggable align="center" autoplayInterval={5000} className={styles.root}
            onActiveIndexChange={(ev, data) => {
                setIndex(data.index)
            }}
        >
            <CarouselViewport>
                <CarouselSlider cardFocus className={styles.root}>
                    {items.map((item, index) => (
                        <BannerCard key={`image-${index}`} item={item}>
                        </BannerCard>
                    ))}
                </CarouselSlider>
            </CarouselViewport>
            {items.length > 1 && <CarouselNavContainer
                layout="overlay-expanded"
                autoplay={{
                    style: { display: "none" },
                    checked: true,
                }}
                next={{
                    className: styles.btn,
                    style: (activeIndex == items.length - 1 || ismobile) ? { display: "none" } : {}
                }}
                prev={{
                    className: styles.btn,
                    style: (activeIndex == 0 || ismobile) ? { display: "none" } : {}
                }}
            >
                <CarouselNav appearance="brand" style={{ background: "transparent", margin: "-30px" }}>
                    {(index) => (
                        <CarouselNavButton aria-label={`Carousel Nav Button ${index}`} className={classes.btn1} />
                    )}
                </CarouselNav>
            </CarouselNavContainer>}
        </Carousel>
        </Suspense>
    )
}