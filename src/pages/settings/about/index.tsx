import BasePage from '@/layout/basePage';
import Image from "next/image";
import { useEffect, useState } from 'react';
import { useI18n } from '@/i18n';
import styles from "./about.module.css";
import LogoIcon from './logoIcon.svg';
import LogoName from "./logoName.svg";
import Sun from "./sun.png";
// import Logo from './logo.png';
import { Button, Persona, Spinner, Subtitle1 } from "@fluentui/react-components";
// import { getVersion } from "@tauri-apps/api/app";
import { ArrowDownload16Filled, ArrowSync16Filled, Checkmark16Filled, ClipboardTextLtr20Filled, Clock20Filled, Info20Filled, People20Filled } from "@fluentui/react-icons";

import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import AurysianYanAvatar from "./avaters/152011199.jpg";
import SearchstarsAvatar from "./avaters/47274115.png";
import sixsixhhAvatar from "./avaters/49398720.jpg";
import lesetongAvatar from "./avaters/84658585.png";
import fangAidenAvatar from "./avaters/47704573.jpeg";
import { UpdateInfo } from '@/components/UpdateDialog/UpdateDialog';
import { BuildInfo } from '@/pages/_app';

// export default async function AboutPage() {
export default function AboutPage() {
    const { t } = useI18n();
    // 应用版本
    const [info, setInfo] = useState<BuildInfo | null>(null)

    const [easterEggCount, setEasterEggCount] = useState<number>(1);
    const showEasterEgg = easterEggCount > 5;
    const [easterEggTime, setEasterEggTime] = useState(-1);
    const [showVideo, setShowVideo] = useState(false);
    const angle = Math.PI * 2 * Math.random()
    const [sunX] = useState<number>(Math.cos(angle) * 800);
    const [sunY] = useState<number>(Math.sin(angle) * 800);
    const [status, setStatus] = useState<'idle' | 'checking' | 'found' | 'newest'>('idle');

    // 团队成员配置
    const teamMembers = [
        {
            name: `Se${Array(easterEggCount).fill("").map(() => (Math.random() > 0.5 || easterEggCount == 1) ? 'a' : "A").join('')}rchstars`,
            role: t('about.roles.nativeFrontend'),
            avatarKey: SearchstarsAvatar
        },
        {
            name: `66h${Array(easterEggCount).fill("").map(() => (Math.random() > 0.5 || easterEggCount == 1) ? 'h' : "H").join('')}`,
            role: t('about.roles.native'),
            avatarKey: sixsixhhAvatar
        },
        {
            name: `leset${Array(easterEggCount).fill("").map(() => (Math.random() > 0.5 || easterEggCount == 1) ? 'o' : "O").join('')}ng`,
            role: t('about.roles.nativeFrontend'),
            avatarKey: lesetongAvatar
        },
        {
            name: `Aurysian${Array(easterEggCount).fill("").map(() => (Math.random() > 0.5 || easterEggCount == 1) ? 'Y' : "y").join('')}an`,
            role: t('about.roles.designerFrontend'),
            avatarKey: AurysianYanAvatar
        },
        {
            name: `FangAi${Array(easterEggCount).fill("").map(() => (Math.random() > 0.5 || easterEggCount == 1) ? 'd' : "D").join('')}en`,
            role: t('about.roles.native'),
            avatarKey: fangAidenAvatar
        }
    ];

    const handleClick = async () => {
        if (status === 'checking') return;
        if (status === "found") return openUrl("https://astrobox.online/download")
        // 正常的检查更新流程
        setStatus('checking');
        const response = await fetch("https://astrobox.online/version.json");
        const data: UpdateInfo = await response.json();
        if (new Date(data.time).getTime() > new Date(info?.BUILD_TIME as string).getTime()) {
            setStatus('found');
            invoke<BuildInfo>("get_build_info").then(info => {
                info.BUILD_TIME = new Date(info.BUILD_TIME).toLocaleString();
                info.VERSION = data.version
                setInfo(info)
            })
        } else {
            setStatus('newest');
        }
    };

    const getButtonContent = () => {
        switch (status) {
            case 'checking':
                return (
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <Spinner size="extra-tiny" appearance="inverted" style={{ filter: window.matchMedia('(prefers-color-scheme: light)').matches ? 'invert(1)' : 'none' }} />
                        {t('about.checking')}
                    </div>
                );
            case 'found':
                return (
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <ArrowDownload16Filled />
                        {t('about.download')}
                    </div>
                );
            case 'newest':
                setTimeout(() => {
                    setStatus('idle');
                }, 2000);
                return (
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <Checkmark16Filled />
                        {t('about.latest')}
                    </div>
                )
            default:
                return (
                    <div style={{ display: 'flex', alignItems: 'center', gap: '8px' }}>
                        <ArrowSync16Filled />
                        {t('about.checkUpdate')}
                    </div>
                );
        }
    };

    useEffect(() => {
        invoke<BuildInfo>("get_build_info").then(info => {
            info.BUILD_TIME = new Date(info.BUILD_TIME).toLocaleString();
            setInfo(info)
        })
    }, [])

    const textColor = status === 'found' ? 'var(--colorNeutralForegroundOnBrand)' : 'var(--colorNeutralForeground1)';

    return (
        <BasePage title={t('about.title')}>
            <main className={styles.main}>
                <div className={styles.logoContainer}>
                    {!showVideo&&<Image
                        src={LogoIcon}
                        alt='Logo'
                        width={268}
                        height={128}
                        className='svg'
                        style={{ position: "absolute", margin: "auto" }}
                    />}
                    {showVideo && <><video autoPlay loop style={{ pointerEvents: "none", background: "transparent", zIndex: 10, marginTop: -90, position: "absolute" }} width={268} height={268}>
                        <source src="/boom.webm" type="video/webm" />
                    </video><audio autoPlay onEnded={() => {setShowVideo(false);}}>
                        <source src="/boom.mp3" type="audio/mp3"/>
                    </audio></>}
                    <Image
                        src={LogoName}
                        alt='Astrobox'
                        width={268}
                        height={128}
                        className='svg'
                        style={{ position: "absolute", margin: "auto", }}
                    />
                    <Image
                        src={Sun}
                        alt='Sun'
                        width={268}
                        height={128}
                        style={{ position: "absolute", margin: "auto", pointerEvents: "none", visibility: showEasterEgg ? "visible" : "hidden", transition: "transform 2s var(--bouncy)", transform: `translate(${!showEasterEgg ? sunX : 0}px,${!showEasterEgg ? sunY : 0}px)` }}
                    />
                </div>
                <div className={styles.aboutContainer}>
                    <div className={styles.versionSection} style={{
                        background: status === 'found' ? 'var(--colorBrandBackground)' : 'var(--cardbackground)'
                    }}>
                        <div className={styles.versionContainer}>
                            <Info20Filled style={{ color: textColor }} />
                            <p className={styles.cardTitle} style={{ color: textColor }}>{status === 'found' ? t('about.foundNew') : t('about.appVersion')}</p>
                        </div>
                        <div className={styles.versionContainer}>
                            <p className={styles.version} style={{ color: textColor }} onClick={() => {
                                if (easterEggTime - Date.now() < -500 && !showEasterEgg) {
                                    setEasterEggCount(1);
                                } else {
                                    setEasterEggCount(easterEggCount + 1);
                                    if (easterEggCount > 5) return;
                                }
                                if (easterEggCount >= 5) setTimeout(() => { setShowVideo(true) }, 100)
                                setEasterEggTime(Date.now());
                            }}>{info?.VERSION}</p>
                            <Button
                                appearance={status === 'found' ? 'primary' : 'transparent'}
                                // appearance='transparent'
                                onClick={handleClick}
                                shape="square"
                                style={{
                                    display: 'flex',
                                    alignItems: 'center',
                                    width: 'fit-content',
                                    minWidth: 'unset',
                                    maxHeight: '16px',
                                    justifyContent: 'center',
                                    pointerEvents: status === 'checking' ? 'none' : 'auto',
                                    padding: '0',
                                    border: 'none',
                                    background: 'transparent',
                                    fontSize: '14px',
                                    paddingLeft: '12px',
                                    paddingRight: '0px',
                                    borderLeft: '2px solid color-mix(in srgb, ' + textColor + ' 40%,transparent)'
                                }}
                            >
                                {getButtonContent()}
                            </Button>
                        </div>
                    </div>
                    <div className={styles.versionSection}>
                        <div className={styles.versionContainer}>
                            <Clock20Filled />
                            <p className={styles.cardTitle}>{t('about.buildDate')}</p>
                        </div>
                        <p className={styles.version}>{info?.BUILD_TIME}</p>
                    </div>
                    <div className={styles.teamSection}>
                        <div className={styles.cardTitle} style={{
                            padding: "var(--spacingVerticalXXS) var(--spacingHorizontalXS)"
                        }}>
                            <People20Filled />
                            <p className={styles.cardTitle}>{t('about.team')}</p>
                        </div>
                        <div className={styles.teamContainer}>
                            {teamMembers.map((member, index) => (
                                <Persona
                                    name={member.name}
                                    secondaryText={member.role}
                                    avatar={{
                                        image: {
                                            src: member.avatarKey.src,
                                        },
                                    }}
                                    size="extra-large"
                                />
                            ))}
                        </div>
                    </div>
                    <div className={styles.teamSection}>
                        <div className={styles.cardTitle} style={{
                            padding: "var(--spacingVerticalXXS) var(--spacingHorizontalXS)"
                        }}>
                            <ClipboardTextLtr20Filled />
                            <p className={styles.cardTitle}>{t('about.changelog')}</p>
                        </div>
                        <div className={styles.content}>
                            <Subtitle1>{t('about.releaseName')}: <b>Helios</b></Subtitle1>
                            <p className={styles.caption}>{t('about.releaseSlogan')}<br /></p>
                            <p className={styles.buildinfo}>
                                COMMIT_HASH:{info?.GIT_COMMIT_HASH}<br />
                                BUILD_USER: {info?.BUILD_USER}
                            </p>
                        </div>
                    </div>
                </div>
                <p className={styles.caption}>{t('about.copyright')}</p>
            </main>
        </BasePage >
    );
};