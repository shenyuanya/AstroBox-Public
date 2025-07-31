import { useI18n } from '@/i18n';
import BasePage from '@/layout/basePage';
import Image from "next/image";
import { useEffect, useState } from 'react';
import styles from "./about.module.css";
import LogoIcon from './logoIcon.svg';
import LogoName from "./logoName.svg";
import Sun from "./sun.png";
// import Logo from './logo.png';
import { Button, Persona, Spinner, Subtitle1 } from "@fluentui/react-components";
// import { getVersion } from "@tauri-apps/api/app";
import { LocalLanguage20Filled, ArrowDownload16Filled, ArrowSync16Filled, Checkmark16Filled, ClipboardTextLtr20Filled, CaretRight20Filled, People20Filled } from "@fluentui/react-icons";

import { UpdateInfo } from '@/components/UpdateDialog/UpdateDialog';
import { BuildInfo } from '@/pages/_app';
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import AurysianYanAvatar from "./avaters/152011199.jpg";
import SearchstarsAvatar from "./avaters/47274115.png";
import fangAidenAvatar from "./avaters/47704573.jpeg";
import sixsixhhAvatar from "./avaters/49398720.jpg";
import lesetongAvatar from "./avaters/84658585.png";

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
                    <div className={styles.buttonContainer}>
                        <div className={`${styles.spinnerContainer} ${styles.found}`}>
                            <Spinner size="extra-tiny" appearance="inverted" style={{ filter: window.matchMedia('(prefers-color-scheme: light)').matches ? 'invert(1)' : 'none' }} />
                        </div>
                        <div className={styles.versionInfoContainer}>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.appVersion')} {info?.VERSION}</p>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.checking')}</p>
                        </div>
                    </div>
                );
            case 'found':
                return (
                    <div className={styles.buttonContainer}>
                        <div className={`${styles.spinnerContainer} ${styles.found}`}>
                            <ArrowDownload16Filled />
                        </div>
                        <div className={styles.versionInfoContainer}>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{info?.VERSION} <CaretRight20Filled /> {t('about.foundNew')}</p>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.download')}</p>
                        </div>
                    </div>
                );
            case 'newest':
                setTimeout(() => {
                    setStatus('idle');
                }, 2000);
                return (
                    <div className={styles.buttonContainer}>
                        <div className={`${styles.spinnerContainer} ${styles.success}`}>
                            <Checkmark16Filled />
                        </div>
                        <div className={styles.versionInfoContainer}>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.appVersion')} {info?.VERSION}</p>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.latest')}</p>
                        </div>
                    </div>
                )
            default:
                return (
                    <div className={styles.buttonContainer}>
                        <div className={styles.spinnerContainer}>
                            <ArrowSync16Filled />
                        </div>
                        <div className={styles.versionInfoContainer}>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.appVersion')} {info?.VERSION}</p>
                            <p className={styles.versionInfoTitle} style={{ color: textColor }}>{t('about.checkUpdate')}</p>
                        </div>
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
                <div className={styles.aboutHeader}>
                    <div className={styles.logoContainer} onClick={() => {
                        if (easterEggTime - Date.now() < -500 && !showEasterEgg) {
                            setEasterEggCount(1);
                        } else {
                            setEasterEggCount(easterEggCount + 1);
                            if (easterEggCount > 5) return;
                        }
                        if (easterEggCount >= 5) setTimeout(() => { setShowVideo(true) }, 100)
                        setEasterEggTime(Date.now());
                    }}>
                        {/* 六次以内点击切换logo显示状态，默认显示 LogoName */}
                        {!showVideo && (
                            (easterEggCount < 6 && easterEggCount % 2 === 1) ? (
                                <Image
                                    src={LogoName}
                                    alt='Astrobox'
                                    width={285}
                                    height={64}
                                    className='svg'
                                    style={{ position: "absolute", margin: "auto" }}
                                />
                            ) : (
                                <Image
                                    src={LogoIcon}
                                    alt='Logo'
                                    width={285}
                                    height={64}
                                    className='svg'
                                    style={{ position: "absolute", margin: "auto" }}
                                />
                            )
                        )}
                        {showVideo && <><video autoPlay loop style={{ pointerEvents: "none", background: "transparent", zIndex: 10, marginTop: -28, marginLeft: -28, position: "absolute" }} width={128} height={128}>
                            <source src="/boom.webm" type="video/webm" />
                        </video><audio autoPlay onEnded={() => { setShowVideo(false); }}>
                                <source src="/boom.mp3" type="audio/mp3" />
                            </audio></>}
                        {/* 移除多余的 LogoName 显示，已在条件判断中处理 */}
                        <Image
                            src={Sun}
                            alt='Sun'
                            width={285}
                            height={64}
                            style={{ position: "absolute", margin: "auto", pointerEvents: "none", visibility: showEasterEgg ? "visible" : "hidden", transition: "transform 2s var(--bouncy)", transform: `translate(${!showEasterEgg ? sunX : 0}px,${!showEasterEgg ? sunY : 0}px)` }}
                        />
                    </div>
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
                            justifyContent: 'center',
                            pointerEvents: status === 'checking' ? 'none' : 'auto',
                            padding: '0',
                            border: 'none',
                            background: 'transparent',
                            fontSize: '14px',
                        }}
                    >

                        {getButtonContent()}
                    </Button>
                </div>
                <div className={styles.aboutContainer}>
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
                                    style={{
                                        minWidth: "calc(50% - 6px)",
                                    }}
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
                            <Subtitle1>{t('about.releaseName')}: Helios</Subtitle1>
                            <p className={styles.caption}>{t('about.releaseSlogan')}<br /></p>
                            {"新功能：\n1. 首页允许根据支持的设备分类以过滤资源（入口：搜索按钮）\n2. 首页允许根据资源的付费类型分类以过滤资源（入口：搜索按钮）\n3. 设置页中现在会显示翻译贡献者了\n4. 设置页中新增“调试窗口”开关，可实时输出应用日志，重启应用生效\n5. 实现插件接口ui.openPageWithUrl\n6. 新增应用内广播系统，用于推送重要通知\n\nBug修复 / 体验增强：\n1. 修复了安卓端上安装完成后自动删除源文件工作异常的问题\n2. 修复了安装本地资源完成后会将本地资源删除的问题\n3. 重构了搜索对话框组件\n4. 优化了首页无限滚动的加载逻辑\n5. 升级了Tauri版本".split('\n').map((line, index) => (
                                <p key={index} style={{ margin: '4px 0' }}>{line}</p>
                            ))}
                            <p className={styles.buildinfo}>
                                COMMIT_HASH:{info?.GIT_COMMIT_HASH}<br />
                                BUILD_USER: {info?.BUILD_USER}<br />
                                BUILD_TIME: {info?.BUILD_TIME}
                            </p>
                        </div>
                    </div>
                </div>
                <p className={styles.caption}>{t('about.copyright')}</p>
            </main>
        </BasePage >
    );
};