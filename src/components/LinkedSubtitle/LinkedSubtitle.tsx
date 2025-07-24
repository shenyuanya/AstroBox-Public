import React from "react";
import { Subtitle1 } from "@fluentui/react-components";
import { ChevronRight20Filled } from "@fluentui/react-icons";
import { useRouter } from "next/router";
import { openUrl } from "@tauri-apps/plugin-opener";
import styles from "./LinkedSubtitle.module.css";

interface LinkedSubtitleProps {
    text: string;
    link: string;
    className?: string;
}

const LinkedSubtitle: React.FC<LinkedSubtitleProps> = ({ text, link, className }) => {
    const router = useRouter();

    const handleClick = async () => {
        try {
            if (link.startsWith("/") || link.startsWith("#")) {
                router.push(link);
            } else {
                await openUrl(link);
            }
        } catch (err) {
            console.error("跳转失败：", err);
        }
    };

    return (
        <Subtitle1
            onClick={handleClick}
            className={`${styles.LinkedSubtitle} ${className || ""}`}
        >
            <span>{text}</span>
            <ChevronRight20Filled className={styles.icon} />
        </Subtitle1>
    );
};

export default LinkedSubtitle;
