import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import ProviderDialog from "@/components/ProviderDialog/ProviderDialog";
import BasePage from "@/layout/basePage";
import { Button, Portal, SearchBox, tokens } from "@fluentui/react-components";
import { Box24Filled, SearchFilled } from "@fluentui/react-icons";
import { AnimatePresence, motion } from "framer-motion";
import { useMemo, useRef, useState } from "react";
import { useI18n } from "@/i18n";
import AppList from "../../../components/home/homeApplist";
import Banner from "../../../components/home/homeBanner";
import styles from "./communityhome.module.css";

export default function CommunityHome() {
    const { t } = useI18n();
    const [dialogOpen, setDialogOpen] = useState(false);
    const hash = useMemo(() => {
        try {
            return JSON.parse(decodeURIComponent(window.location.hash.slice(1)));
        } catch (error) {
            return {};
        }
    }, [])
    const [search, setSearch] = useState(hash.search as string);

    return (<>
        <BasePage
            title={t('home.title')}
            // externalTitle={
            //     <div className={styles.repoBtn} onClick={() => setDialogOpen(true)}>
            //         <Box24Filled style={{ opacity:0.75, marginLeft: "10px" }} />
            //     </div>
            // }
            action={<div style={{ display: "flex", flexDirection: "row" }}>
                <AppleButtonWrapper padding={5}>
                    <Button icon={<Box24Filled style={{ opacity: 0.75 }} />} appearance="transparent" className={styles.repoBtn} onClick={() => setDialogOpen(true)}></Button>
                </AppleButtonWrapper>
                <SearchDialog onDone={setSearch} defaultValue={search} />
            </div>}
            style={{ overflow: "visible" }}
        >
            <Banner />
            <AppList search={search}/>
        </BasePage >
        <ProviderDialog open={dialogOpen} onClose={() => setDialogOpen(false)} />
    </>)
}
function SearchDialog({ onDone, defaultValue }: { onDone: (search: string) => void, defaultValue: string }) {
    const searchRef = useRef<HTMLInputElement>(null);
    const [open, setOpen] = useState(false)
    const { t } = useI18n();
    const handleClick = () => {
        const searchValue = searchRef.current?.value || "";
        onDone(searchValue);
        const hash = (() => {
            try {
                return JSON.parse(decodeURIComponent(window.location.hash.slice(1)));
            } catch (error) {
                return {};
            }
        })()
        hash.search = searchValue;
        window.location.hash = JSON.stringify(hash);
    }
    return <>
        <AppleButtonWrapper padding={5}>
            <Button icon={<SearchFilled style={{ opacity: .75 }} />} appearance="transparent" onClick={() => setOpen(true)} />
        </AppleButtonWrapper>
        <Portal mountNode={{ className: "portal" }}>
            <AnimatePresence>
                {open && <motion.div
                    initial={{scaleX: 0, opacity: 0}}
                    animate={{scaleX: 1, opacity: 1}}
                    exit={{scaleX: 0, opacity: 0}}
                    style={{ transformOrigin: "right", position: "absolute", top: "var(--safe-area-inset-top)", boxSizing: "border-box", width: "100%", padding: "16px", display: "flex", justifyContent: "flex-end" }}>
                    <div className="card" style={{ width: "100%", maxWidth: "468px", padding: 2, background: tokens.colorNeutralCardBackground }}>
                        <SearchBox placeholder={t('home.search')} style={{ width: "100%" }} ref={searchRef}
                            defaultValue={defaultValue}
                            onBlur={(e) => {
                                if (e.target.value == "") setOpen(false);
                                handleClick()
                            }}
                            onKeyDown={(e) => {
                                if (e.key == "Escape") setOpen(false);
                                if (e.key == "Enter") handleClick();
                            }}
                            autoFocus
                        />
                    </div>
                </motion.div>}
            </AnimatePresence>
        </Portal>
    </>
}