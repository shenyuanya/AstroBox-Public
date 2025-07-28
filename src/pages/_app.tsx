import AnimatedLayout from "@/animation/AnimatedLayout";
import { NavDirectionProvider } from "@/animation/NavDirectionContext";
import { useSystemTheme } from "@/hooks/useSystemTheme";
import { I18nProvider, useI18n } from "@/i18n";
import '@capacitor-community/safe-area';
import { FluentProvider, webDarkTheme, webLightTheme } from "@fluentui/react-components";
import type { AppProps } from "next/app";
import { useEffect, useState } from "react";

import AddDeviceFromQr from "@/components/EventHandlers/addDeviceFromQr";
import DragToPush from "@/components/EventHandlers/drag";
import HomeNav from "@/components/HomeNav/HomeNav";
import QueueTrigger from "@/components/Queue/QueueTrigger";
import {
  CheckBlePermissionWithAlert,
  CheckLocationPermissionWithAlert,
  RequestLocationPermission
} from "@/device/permission";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import useIsMobile from "@/hooks/useIsMobile";
import { isNav, navItems } from "@/router/nav";
import { invoke } from "@tauri-apps/api/core";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { version as osVersion, platform } from "@tauri-apps/plugin-os";
import "./root.css";

// Listeners
import DebugWindow from "@/components/Debug/DebugWindow";
import { DisclaimerDialog } from "@/components/disclaimer/disclamierDialog";
import UpdateDialog, { UpdateInfo } from "@/components/UpdateDialog/UpdateDialog";
import useToast, { makeError, ToastSurface } from "@/layout/toast";
import { initOpenResourceListener } from "@/listeners/resources";
import logger from "@/log/logger";
import Head from "next/head";
import BoardcastDialog, { BoardcastInfo } from "@/components/BoardcastDialog/BoardcastDialog";

const makeFluentTheme = (mode: 'light' | 'dark') => {
  return mode === 'light' ? webLightTheme : webDarkTheme;
}
function listenCloseEvent() {
  getCurrentWindow().onCloseRequested(async () => {
    await invoke("cleanup_before_exit");
  });
}

export interface BuildInfo {
  GIT_COMMIT_HASH: string,
  BUILD_TIME: string,
  BUILD_USER: string,
  VERSION: string,
}

function App({ Component, pageProps }: AppProps) {
  const { t } = useI18n()
  const mode = useSystemTheme();
  const [baseStyles, setBaseStyles] = useState<any>({ "background": mode == "dark" ? "#1b1a19" : "#fff" });
  const [background, setBackground] = useState("");
  const router = useAnimatedRouter();
  const ismobile = useIsMobile();
  const [currentNav, setCurrentNav] = useState("/");
  const [hasMounted, setHasMounted] = useState(false);
  const [debugEnabled, setDebugEnabled] = useState(process.env.NODE_ENV === 'development');
  const { dispatchToast } = useToast();

  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(null);
  const [showUpdateDialog, setShowUpdateDialog] = useState(false);
  const [boardcastQueue, setBoardcastQueue] = useState<BoardcastInfo[]>([]);
  const [currentBoardcast, setCurrentBoardcast] = useState<BoardcastInfo | null>(null);
  const [showBoardcastDialog, setShowBoardcastDialog] = useState(false);

  useEffect(() => {
    setHasMounted(true);
  }, []);

  useEffect(() => {
    invoke<any>('app_get_config')
      .then(cfg => {
        if (typeof cfg.debug_window === 'boolean') {
          setDebugEnabled(cfg.debug_window);
        }
      })
      .catch(() => { });
  }, []);

  // 启动时检查更新
  useEffect(() => {
    const checkForUpdates = async () => {
      try {
        const localInfo = await invoke<BuildInfo>("get_build_info");
        const response = await fetch("https://astrobox.online/version.json", { cache: "no-store" });
        const remoteInfo: UpdateInfo = await response.json();

        const ignoredVersion = localStorage.getItem('ignoredUpdateVersion');

        if (new Date(remoteInfo.time).getTime() > new Date(localInfo.BUILD_TIME).getTime()) {
          if (remoteInfo.version !== ignoredVersion) {
            setUpdateInfo(remoteInfo);
            setShowUpdateDialog(true);
          }
        }
      } catch (error) {
        logger.error("Failed to check for updates:", error);
      }
    };

    checkForUpdates();
  }, []);

  useEffect(() => {
    const checkForBoardcasts = async () => {
      try {
        const response = await fetch("https://astrobox.online/boardcasts.json", { cache: "no-store" });
        const all = await response.json();
        const popups = (all as BoardcastInfo[]).filter(item => !!item.popup);
        setBoardcastQueue(popups);
      } catch (error) {
        logger.error("Failed to check for boardcasts:", error);
      }
    }
    checkForBoardcasts();
  }, []);

  useEffect(() => {
    if (boardcastQueue.length > 0 && !showUpdateDialog) {
      setCurrentBoardcast(boardcastQueue[0]);
      setShowBoardcastDialog(true);
    } else {
      setShowBoardcastDialog(false);
      setCurrentBoardcast(null);
    }
  }, [boardcastQueue, showUpdateDialog]);

  const handleCloseBoardcast = () => {
    setBoardcastQueue(prev => prev.slice(1));
    setShowBoardcastDialog(false);
  };

  // Register listeners
  useEffect(() => {
    initOpenResourceListener(router);
  }, [])

  useEffect(() => {
    const handleContextMenu = (e: MouseEvent) => {
      if (process.env.NODE_ENV === 'development') {
        return;
      }
      e.preventDefault();
    };
    document.addEventListener('contextmenu', handleContextMenu);
    return () => {
      document.removeEventListener('contextmenu', handleContextMenu);
    };
  }, []);

  useEffect(() => {
    const plat = platform();
    const version = osVersion();

    if (plat === "windows") {
      if (parseInt(version.split(".")[2]) >= 22000) {
        setBaseStyles({ "background": "transparent", "height": "100%", "--border-radius": "10px 0 0 0", "--border-width": "1px 0 0 1px" })
        setBackground("win11")
        return
      }
    }
    setBaseStyles({ "background": mode !== "dark" ? "#fff" : "#1b1a19" })
  }, [mode]);

  const isNavPage = isNav(router.pathname);
  const visible = !ismobile || isNavPage;

  useEffect(() => {
    if (isNavPage) {
      setCurrentNav(router.pathname);
    }
  }, [router.pathname, isNavPage])

  const handleNavItemClick = (name: string) => {
    if (name === router.pathname) return;
    router.replace(name);
  };

  const handleIgnoreUpdate = (version: string) => {
    localStorage.setItem('ignoredUpdateVersion', version);
  };

  useEffect(() => {
    setTimeout(async () => {
      try {
        await CheckBlePermissionWithAlert();
        await CheckLocationPermissionWithAlert();
        await RequestLocationPermission();
      } catch (error) {
        if (error instanceof Error) {
          const [key, ...rest] = error.message.split(":");
          const detail = rest.join(":");
          makeError(dispatchToast, detail ? `${t(key)}:${detail}` : t(key));
        }
      }
    }, 500);
    listenCloseEvent();
    window.onpopstate = () => {
      if (isNav(window.location.pathname)) setCurrentNav(window.location.pathname);
    }
  }, []);

  if (!hasMounted) {
    return (
      <div className="baseBody" style={baseStyles}>
        <I18nProvider>
          <FluentProvider theme={webDarkTheme} style={{ background: "transparent", height: "100%", overflow: "hidden" }}>
          </FluentProvider>
        </I18nProvider>
      </div>
    );//防止第一次加载时闪烁
  }
  const disclaimerOpen = localStorage.getItem("disclaimerAccepted") !== "true";

  return (
    <div className="baseBody" style={baseStyles}>
      <Head>
        <meta name="referrer" content="no-referrer" />
        <meta name="viewport" content="viewport-fit=cover ,width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no" />
      </Head>
      <I18nProvider>
        <FluentProvider theme={makeFluentTheme(mode)} style={{ background: "transparent", height: "100%", overflow: "hidden" }}>
          <NavDirectionProvider>
            <ToastSurface />
            <QueueTrigger />
            <AddDeviceFromQr />
            <DragToPush />
            <DisclaimerDialog defaultOpen={disclaimerOpen} />
            <UpdateDialog
              open={showUpdateDialog}
              onClose={() => setShowUpdateDialog(false)}
              updateInfo={updateInfo}
              onIgnore={handleIgnoreUpdate}
            />
            <BoardcastDialog
              open={showBoardcastDialog && !showUpdateDialog}
              onClose={handleCloseBoardcast}
              boardcastInfo={currentBoardcast}
            />
            <div className="layout-root">
              {
                (visible) && <div className="app-nav-wrapper">
                  <HomeNav navItems={navItems} currentNavItem={currentNav} onNavItemClick={handleNavItemClick} />
                </div>
              }

              <AnimatedLayout>
                <main className={`app-content-wrapper main-content ${background}`}>
                  <Component {...pageProps} currentNav={currentNav} />
                </main>
              </AnimatedLayout>

              {debugEnabled && <DebugWindow></DebugWindow>}
            </div>
          </NavDirectionProvider>
        </FluentProvider>
      </I18nProvider>
    </div>
  );
}

export default App;