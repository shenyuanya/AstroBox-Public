import { registerOpenFileListener } from "@/device/install";
import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import useIsMobile from "@/hooks/useIsMobile";
import { isNav } from "@/router/nav";
import { useDownloadQueue, useInstallQueue } from "@/taskqueue/queue";
import {
  CounterBadge,
  makeStyles,
  Text,
  tokens,
  ProgressBar,
} from "@fluentui/react-components";
import {
  AppsListDetailRegular,
  ChevronUpRegular,
  DismissRegular,
} from "@fluentui/react-icons";
import { getCurrentWindow, ProgressBarStatus } from "@tauri-apps/api/window";
import { AnimatePresence, motion } from "framer-motion";
import React, { memo, useEffect, useRef, useState } from "react";
import styles from "./queue.module.css";
import QueueWindow from "./QueueWindow";
import { useI18n } from "@/i18n";

const useStyles = makeStyles({
  pcTriggerContainer: {
    position: "fixed",
    left: "0px",
    bottom: "0px",
    zIndex: 1300,
  },
  pcButtonWithBadge: {
    position: "relative",
    cursor: "pointer",
  },
  pcBadge: {
    position: "absolute",
    // top: tokens.spacingVerticalXXS,
    top: "1px",
    right: tokens.spacingHorizontalXS,
  },

  mobileTriggerContainer: {
    position: "fixed",
    left: tokens.spacingHorizontalM,
    right: tokens.spacingHorizontalM,
    height: "48px",
    zIndex: 1300,
    background: "color-mix(in oklch,var(--colorNeutralBackground1), transparent 10%)",
    backdropFilter: "blur(30px)",
    borderRadius: tokens.borderRadiusXLarge,
    boxShadow: tokens.shadow16,
    cursor: "pointer",
    display: "flex",
    alignItems: "center",
    boxSizing: "border-box",
    transition: "bottom 0.35s cubic-bezier(0.25, 0.8, 0.25, 1)",
  },
  mobileTriggerContent: {
    display: "flex",
    alignItems: "center",
    width: "100%",
    color: tokens.colorNeutralForeground1,
    paddingLeft: tokens.spacingHorizontalL,
    paddingRight: tokens.spacingHorizontalL,
    overflow: "hidden",
  },
  mobileTriggerText: {
    flexGrow: 1,
    marginLeft: tokens.spacingHorizontalM,
    overflow: "hidden",
    textOverflow: "ellipsis",
    whiteSpace: "nowrap",
  },
});
interface TriggerProps {
  open: boolean;
  count: number;
  classes: ReturnType<typeof useStyles>;
  running?: boolean;
  progress?: number;
  onClick: () => void;
  hasNav: boolean;
}

const PcTrigger: React.FC<TriggerProps> = memo(
  ({ open, count, classes, running, progress, onClick, hasNav }) => {
    const { t } = useI18n();
    const badgeVariants = {
      initial: { scale: 0, opacity: 0 },
      animate: {
        scale: 1,
        opacity: 1,
        transition: { type: "spring" as const, stiffness: 260 },
      },
      exit: { scale: 0, opacity: 0, transition: { duration: 0.15 } },
    };
    if (!running) progress = 100;

    return (
      <motion.div
        className={classes.pcTriggerContainer}
        initial={{ opacity: 0, scale: 0.6 }}
        animate={{ opacity: 1, scale: 1 }}
        transition={{
          duration: 0.3,
          ease: [0.25, 0.8, 0.25, 1] as [number, number, number, number],
        }}
        onClick={onClick}
      >
        <div
          className={styles.pcTrigger}
          style={{ backgroundColor: open ? "color-mix(in srgb, var(--colorBrandBackgroundInverted) 5%, transparent)" : "transparent" }}
        >
          <div className={styles.triggerProgressContainer} style={{ opacity: running ? 1 : 0, }}>
            <div
              className={styles.triggerProgress}
              style={{ height: `${progress}%` }}
            ></div>
          </div>
          {!open ? (
            <AppsListDetailRegular
              fontSize={22}
              // style={{ margin: running ? 10 : "auto" }}
              className={styles.triggerIcon}
            />
          ) : (
            <>
              <DismissRegular
                fontSize={22}
                // style={{ margin: running ? 10 : "auto" }}
                className={styles.triggerIcon}
              />
            </>
          )}
          {!running ? (
            <p className={styles.navLabel}>
              {t('queue.titleShort')}
            </p>
          ) : (
            <p className={styles.navLabel}>
              {progress?.toFixed(0)}%
            </p>
          )}
          {/* {running && (
            <Caption1 style={{ margin: 10 }}>{progress?.toFixed(2)}%</Caption1>
          )} */}
          <AnimatePresence>
            {!open && count > 0 && (
              <motion.div
                style={{ pointerEvents: "none" }}
                key={count}
                className={classes.pcBadge}
                variants={badgeVariants}
                initial="initial"
                animate="animate"
                exit="exit"
              >
                <CounterBadge count={count} size="small" color="danger" />
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </motion.div>
    );
  },
);
PcTrigger.displayName = "PcTrigger";

const MobileTrigger: React.FC<TriggerProps> = memo(
  ({ open, count, classes, running, progress = 100, onClick, hasNav }) => {
    const { t } = useI18n();
    if (open) return null;
    if (!running) progress = 0;
    const bottom = hasNav
      ? `calc(env(safe-area-inset-bottom, 0px) + 64px)`
      : `calc(env(safe-area-inset-bottom, 0px) + 8px)`;
    return (
      <motion.div
        className={classes.mobileTriggerContainer}
        onClick={onClick}
        role="button"
        tabIndex={0}
        aria-expanded={open}
        initial={{ y: 64, opacity: 0, bottom }}
        animate={{ y: 0, opacity: 1, bottom }}
        transition={{
          y: { duration: 0.35, ease: [0.25, 0.8, 0.25, 1] as [number, number, number, number] },
          opacity: { duration: 0.35, ease: [0.25, 0.8, 0.25, 1] as [number, number, number, number] },
          bottom: { duration: 0.35, ease: [0.25, 0.8, 0.25, 1] as [number, number, number, number] },
        }}
      >
        {/* <div
          className={styles.triggerProgress}
          style={{
            width: `${progress}%`,
            borderRadius: tokens.borderRadiusXLarge,
          }}
        ></div> */}
        <div className={classes.mobileTriggerContent}>
          <AppsListDetailRegular fontSize={20} />
          <Text weight="medium" className={classes.mobileTriggerText}>
            {count > 0 ? `${t('queue.viewTasks')} (${count})` : t('queue.taskQueue')}
          </Text>
          <ChevronUpRegular fontSize={20} />
          <ProgressBar
            className={styles.container}
            shape="rounded"
            // thickness="large"
            value={progress / 100}
            style={{ opacity: running ? 1 : 0, transition: "opacity 0.3s ease", position: "absolute", right: "5px", left: "5px", bottom: "1px", width: "calc(100% - 10px)" }}


          />
        </div>
      </motion.div>
    );
  },
);
MobileTrigger.displayName = "MobileTrigger";

/* ---------------- 主组件 ---------------- */
export default function QueueTrigger() {
  let {
    items: downloadItems,
    progress: downloadProgress,
    status: downloadStatus,
  } = useDownloadQueue();
  let {
    items: installItems,
    progress: installProgress,
    status: installStatus,
  } = useInstallQueue();
  const items = [...downloadItems, ...installItems];

  if (downloadStatus == "pending") downloadProgress = 0;
  if (installStatus == "pending") installProgress = 0;

  const progress =
    ((downloadProgress + installProgress) /
      ((downloadStatus == "pending" ? 1 : 0) +
        (installStatus == "pending" ? 1 : 0))) *
    100;

  const hasItems = items.length > 0;
  const running = downloadStatus !== "pending" || installStatus !== "pending";

  useEffect(() => {
    getCurrentWindow().setProgressBar({ progress: Math.round(progress), status: running ? ProgressBarStatus.Normal : ProgressBarStatus.None });
  }, [progress])

  const rawIsMobile = useIsMobile();
  const router = useAnimatedRouter();
  const [open, setOpen] = useState(false);
  const classes = useStyles();
  const { t } = useI18n();

  const [clientSideIsMobile, setClientSideIsMobile] = useState(false);
  const [hasMounted, setHasMounted] = useState(false);
  const hasNav = !clientSideIsMobile || isNav(router.pathname);

  useEffect(() => {
    setClientSideIsMobile(rawIsMobile);
    setHasMounted(true);
  }, [rawIsMobile]);

  const prevCount = useRef(items.length);

  useEffect(() => {
    if (!hasItems && open) setOpen(false);
    prevCount.current = items.length;
  }, [hasItems, items.length, open]);

  useEffect(() => {
    let unregister = registerOpenFileListener();
    return () => {
      unregister.then((unreg) => {
        unreg();
      });
    };
  }, []);

  const handleToggle = () => setOpen((v) => !v);

  if (!hasMounted) return null;

  return (
    <>
      {hasItems &&
        (clientSideIsMobile ? (
          <MobileTrigger
            open={open}
            count={items.length}
            classes={classes}
            onClick={handleToggle}
            running={running}
            progress={progress}
            hasNav={hasNav}
          />
        ) : (
          <PcTrigger
            open={open}
            count={items.length}
            classes={classes}
            onClick={handleToggle}
            running={running}
            progress={progress}
            hasNav={hasNav}
          />
        ))}

      <AnimatePresence mode="wait" initial={false}>
        {open && (
          <QueueWindow
            key={clientSideIsMobile ? "mobile-queue" : "pc-queue"}
            isMobile={clientSideIsMobile}
            onRequestClose={() => setOpen(false)}
          />
        )}
      </AnimatePresence>
    </>
  );
}
