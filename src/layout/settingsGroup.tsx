import { Title3 } from "@fluentui/react-components";
import { motion } from "framer-motion";
import style from "./settingsGroup.module.css";

export default function SettingsGroup({ children, title }: { children?: React.ReactNode, title: string }) {
    return (
        <motion.div
            initial={{ opacity: 0, y: 10 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: 10 }}
            className={style.warpper}>
            <Title3 as="h3" style={{ fontSize: 16, fontWeight: "var(--fontWeightMedium)", margin: "0px 6px", lineHeight: "20px" }}>{title}</Title3>
            {children}
        </motion.div>
    )
}