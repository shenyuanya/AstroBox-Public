import { useAnimatedRouter } from "@/hooks/useAnimatedRouter";
import { isNav } from "@/router/nav";
import { useDownloadQueue, useInstallQueue } from "@/taskqueue/queue";
import { TaskItem } from "@/taskqueue/tasklist";
import {
    Button,
    makeStyles,
    tokens
} from "@fluentui/react-components";
import { Dismiss24Regular, SendRegular, StopRegular } from "@fluentui/react-icons";
import { AnimatePresence, motion } from "framer-motion";
import QueueCard from "./QueueCard";
import { useI18n } from "@/i18n";

const useStyles = makeStyles({
    motionDivBase: {
        position: "fixed",
        display: "flex",
        padding: "6px",
        flexDirection: "column",
        overflow: "hidden",
        zIndex: 1000,
        pointerEvents: "auto",
        boxSizing: "border-box",
        backdropFilter: "blur(30px)",
        overflowY: "auto",
    },

    pcWindow: {
        left: "78px",
        bottom: "6px",
        width: "400px",
        maxHeight: "calc(min(600px, 70vh))",
        borderRadius: tokens.borderRadiusLarge,
        backgroundColor: "color-mix(in oklch,var(--colorNeutralBackground1) 90%, transparent)",
        boxShadow: "0 0 4px rgba(0, 0, 0, 0.24), 0 6px 12px rgba(0, 0, 0, 0.28)",
    },

    mobileWindow: {
        left: tokens.spacingHorizontalM,
        right: tokens.spacingHorizontalM,
        minHeight: "250px",
        maxHeight: "85vh",
        borderRadius: tokens.borderRadiusXLarge,
        backgroundColor: "color-mix(in oklch,var(--colorNeutralBackground1), transparent 10%)",
        boxShadow: "0 -4px 16px rgba(0 0 0 / 0.35)",
        transition: "bottom 0.35s cubic-bezier(0.25, 0.8, 0.25, 1)",
    },

    queueWindowHeader: {
        display: "flex",
        flexDirection: "row",
        justifyContent: "space-between",
        alignItems: "center",
        padding: "4px 4px 0"
    },

});

const pcVariants = {
    initial: { opacity: 0, scale: 0.9, x: -20, y: 20 },
    animate: {
        opacity: 1,
        scale: 1,
        x: 0,
        y: 0,
        transition: {
            duration: 0.25,
            ease: [0.1, 0.8, 0.2, 1] as [number, number, number, number]
        },
    },
    exit: {
        opacity: 0,
        scale: 0.9,
        x: -20,
        y: 20,
        transition: {
            duration: 0.2,
            ease: [0.76, 0, 0.24, 1] as [number, number, number, number]
        },
    }
};

const mobileVariants = {
    initial: { y: "100%" },
    animate: {
        y: "0%",
        transition: {
            duration: 0.3,
            ease: [0.25, 0.8, 0.25, 1] as [number, number, number, number]
        }
    },
    exit: {
        y: "100%",
        transition: {
            duration: 0.25,
            ease: [0.76, 0, 0.24, 1] as [number, number, number, number]
        }
    },
};

interface QueueWindowProps {
    isMobile: boolean;
    onRequestClose: () => void;
}

export default function QueueWindow({ isMobile, onRequestClose }: QueueWindowProps) {
    const router = useAnimatedRouter();
    const classes = useStyles();
    const { t } = useI18n();

    const bottom = (isNav(router.pathname) && isMobile) ? "calc(env(safe-area-inset-bottom, 0px) + 64px)" : "calc(env(safe-area-inset-bottom, 0px) + var(--spacingHorizontalM))";

    const currentVariants = isMobile ? mobileVariants : pcVariants;
    const motionDivClassName = `${classes.motionDivBase} ${isMobile ? classes.mobileWindow : classes.pcWindow}`;

    return (
        <motion.div
            layout
            key={isMobile ? "mobile-queue" : "pc-queue"} /* 保证 exit 动画生效 :contentReference[oaicite:0]{index=0} */
            className={motionDivClassName}
            variants={currentVariants}
            initial="initial"
            animate="animate"
            exit="exit"
            style={{ transformOrigin: isMobile ? "bottom center" : "bottom left", gap: 10, bottom: isMobile ? bottom : undefined }}
            role="dialog"
            aria-modal="true"
            onClick={(e) => e.stopPropagation()}
        >
            <div className={classes.queueWindowHeader}>
                <h2 style={{ margin: "auto 0", paddingLeft: 4 }}>{t('queue.title')}</h2><Button appearance="transparent" icon={<Dismiss24Regular />} onClick={onRequestClose} />
            </div>
            <DownloadQueue />
            <InstallQueue />
        </motion.div>
    );
}
function DownloadQueue() {
    const { t } = useI18n();
    const classes = useStyles();
    const { items, start, remove, status, stop } = useDownloadQueue();
    return <div>
        <div className={classes.queueWindowHeader}><h3 style={{ margin: 0, paddingLeft: 4 }}>{t('queue.downloadQueue')}</h3>{status == "running" ? <Button icon={<StopRegular />} onClick={() => stop()}></Button> : <Button disabled={status == "stopping"} icon={<SendRegular />} onClick={() => start()}></Button>}</div>
        <QueueList list={items} onCancel={(id) => remove(id)}></QueueList>
    </div>
}
function InstallQueue() {
    const { t } = useI18n();
    const classes = useStyles();
    const { items, start, remove, status, stop } = useInstallQueue();
    return <div>
        <div className={classes.queueWindowHeader}><h3 style={{ margin: 0, paddingLeft: 4 }}>{t('queue.installQueue')}</h3>{status == "running" ? <Button icon={<StopRegular />} onClick={() => stop()}></Button> : <Button disabled={status == "stopping"} icon={<SendRegular />} onClick={() => start()}></Button>}</div>
        <QueueList list={items} onCancel={(id) => remove(id)}></QueueList>
    </div>
}
function QueueList({ list, onCancel }: { list: TaskItem[]; onCancel: (id: string) => void }) {
    const { t } = useI18n();
    return (
        <AnimatePresence mode="popLayout">
            {list.length ? list.map((task) => (
                <motion.div
                    key={task.id}
                    initial={{ opacity: 0 }}
                    animate={{ opacity: 1 }}
                    exit={{ opacity: 0 }}
                    layout
                >
                    <QueueCard
                        item={task}
                        onCancel={() => onCancel(task.id)}
                    ></QueueCard>
                </motion.div>

            )) : (
                <div style={{
                    padding: "0 8px 0 8px"
                }}>{t('queue.noTasks')}</div>
            )}
        </AnimatePresence>
    );
}