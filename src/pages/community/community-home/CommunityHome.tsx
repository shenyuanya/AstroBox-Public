import { providerManager } from "@/community/manager";
import AppleButtonWrapper from "@/components/appleButtonWapper/appleButtonWapper";
import WinDropdown from "@/components/settings/winDropdown";
import useDeviceMap from "@/hooks/useDeviceMap";
import { useI18n } from "@/i18n";
import BasePage from "@/layout/basePage";
import logger from "@/log/logger";
import { Provider, ProviderState } from "@/plugin/types";
import { Button, Portal, SearchBox, tokens } from "@fluentui/react-components";
import { CheckmarkCircleFilled, DismissCircleFilled, SearchFilled } from "@fluentui/react-icons";
import { AnimatePresence, motion } from "framer-motion";
import { useEffect, useMemo, useRef, useState } from "react";
import useSWR from "swr";
import AppList from "../../../components/home/homeApplist";
import Banner from "../../../components/home/homeBanner";

const PROVIDER_STORAGE_KEY = "community_provider";

export default function CommunityHome() {
    const { t } = useI18n();
    const { providers, errors: providersErrors, lastUpdated } = providerManager.useProviders();
    const [currentProvider, setCurrentProvider] = useState<number>()
    const hash = useMemo(() => {
        try {
            return JSON.parse(decodeURIComponent(window.location.hash.slice(1)));
        } catch (error) {
            return {};
        }
    }, [])
    const [search, setSearch] = useState(hash.search as string);
    const [categories, setCategories] = useState((hash.categories as string[]) ?? []);

    useEffect(() => {
        if (providers.length === 0) return;

        let selectedName: string | null = null;
        selectedName = localStorage.getItem(PROVIDER_STORAGE_KEY) || "official";

        let provider = providers.find(e => e.name === selectedName!);
        if (!provider) {
            provider = providers.find(p => !providersErrors.some(e => e.name === p.name)) || providers[0];
        }

        const index = providers.indexOf(provider);
        setCurrentProvider(index);
        localStorage.setItem(PROVIDER_STORAGE_KEY, provider.name);
    }, [providers, lastUpdated, search]);

    const handleProviderChange = (provider: number) => {
        setCurrentProvider(provider);
        setCategories([]);
        localStorage.setItem(PROVIDER_STORAGE_KEY, providers[provider].name);
    }

    return (<>
        <BasePage
            title={t('home.title')}
            // externalTitle={
            //     <div className={styles.repoBtn} onClick={() => setDialogOpen(true)}>
            //         <Box24Filled style={{ opacity:0.75, marginLeft: "10px" }} />
            //     </div>
            // }
            action={<div style={{ display: "flex", flexDirection: "row" }}>
                <ProviderSelector providers={providers} currentProvider={currentProvider ?? 99} setCurrentProvider={handleProviderChange} />
                <SearchDialog categories={categories} onDone={(s, c) => { setSearch(s); setCategories(c); }} defaultValue={search} provider={providers[currentProvider ?? 99]} />
            </div>}
            style={{ overflow: "visible" }}
        >
            <Banner />
            {/* @ts-ignore */}
            <AppList search={search} provider={providers[currentProvider ?? 99]} categories={categories} />
        </BasePage >
    </>)
}
function SearchDialog({ categories, onDone, defaultValue, provider }: { categories: string[], onDone: (search: string, categories: string[]) => void, defaultValue: string, provider?: Provider }) {
    const searchRef = useRef<HTMLInputElement>(null);
    const [open, setOpen] = useState(false)
    const { t } = useI18n();
    const { data: options } = useSWR("_filter_" + provider?.name, () => provider?.getCategories());
    const [selected, setSelected] = useState<any>(categories.reduce((pre, cur) => { return { ...pre, [cur]: true } }, {}));
    const [lastProvider, setLastProvider] = useState(provider?.name);
    useEffect(() => {
        if (lastProvider !== provider?.name) {
            setSelected({});
            setLastProvider(provider?.name);
        }
    }, [provider?.name])
    const handleClick = () => {
        const searchValue = searchRef.current?.value || "";
        const categories = [];
        for (const category of Object.keys(selected)) {
            if (selected[category]) categories.push(category);
        }
        onDone(searchValue, categories);
        const hash = (() => {
            try {
                return JSON.parse(decodeURIComponent(window.location.hash.slice(1)));
            } catch (error) {
                return {};
            }
        })()
        hash.search = searchValue;
        hash.categories = categories;
        window.location.hash = JSON.stringify(hash);
        setOpen(false);
    }
    return <>
        <AppleButtonWrapper padding={5}>
            <Button icon={<SearchFilled style={{ opacity: .75 }} />} appearance="transparent" onClick={() => setOpen(true)} />
        </AppleButtonWrapper>
        <Portal mountNode={{ className: "portal" }}>
            <AnimatePresence>
                {open && <motion.div
                    style={{ display: "flex", flexDirection: "column", position: "absolute", top: "var(--safe-area-inset-top)", width: "100vw", height: "100vh", alignItems: "flex-end", backgroundColor: "rgba(0,0,0,0.5)" }}
                    tabIndex={1}
                    onClick={(e) => { handleClick(); }}
                    onKeyDown={(e) => {
                        if (e.key == "Escape") setOpen(false);
                        if (e.key == "Enter") handleClick();
                    }}
                    initial={{ opacity: 0, backdropFilter: "blur(0px)" }}
                    animate={{ opacity: 1, backdropFilter: "blur(15px)" }}
                    exit={{ opacity: 0, backdropFilter: "blur(0px)" }}
                >
                    <motion.div
                        initial={{ scaleX: 0, opacity: 0 }}
                        animate={{ scaleX: 1, opacity: 1 }}
                        exit={{ scaleX: 0, opacity: 0 }}
                        style={{ transformOrigin: "right", boxSizing: "border-box", padding: "16px 16px 0 16px", width: "100vw", justifyContent: "end", display: "flex", pointerEvents: "fill" }}>
                        <div className="card" style={{ width: "100%", maxWidth: "468px", padding: 2, background: tokens.colorNeutralCardBackground }}>
                            <SearchBox placeholder={t('home.search')} style={{ width: "100%" }} ref={searchRef}
                                defaultValue={defaultValue}
                                onClick={(e) => { e.stopPropagation(); }}
                                autoFocus
                            />
                        </div>
                    </motion.div>
                    <div style={{ padding: "5px 16px", alignItems: "flex-end", display: "flex", flexDirection: "row-reverse", maxWidth: "468px", flexWrap: "wrap", gap: 5, overflow: "hidden" }}>
                        {options && options.map((option, index) => {
                            return <motion.div
                                key={option}
                                initial={{ opacity: 0, y: -40, x: 40, scale: 0.5 }}
                                animate={{ opacity: 1, y: 0, x: 0, scale: 1 }}
                                exit={{ opacity: 0, y: -40, x: 40, scale: 0.5 }}
                                transition={{ delay: index * 0.02 }}
                                style={{ transformOrigin: "top right" }}
                            >
                                <Category option={option} selected={selected[option]} setSelected={(e) => { setSelected({ ...selected, [option]: !selected[option] }); }} offical={provider?.name == "official"}></Category>
                            </motion.div>
                        })}
                    </div>
                </motion.div>}
            </AnimatePresence>
        </Portal>
    </>
}
function ProviderSelector({ providers, setCurrentProvider, currentProvider }: { providers: Provider[], setCurrentProvider: (index: number) => void, currentProvider: number }) {
    const [states, setStates] = useState<Record<string, ProviderState>>({});
    const { t } = useI18n();

    useEffect(() => {
        let mounted = true;

        const fetchStates = async () => {
            const entries = await Promise.all(
                providers.map(async (p) => {
                    try {
                        return [p.name, await p.getState()];
                    } catch (err) {
                        logger.error("Get state failed", err);
                        return [p.name, 'Error'];
                    }
                })
            );
            if (mounted) {
                setStates(Object.fromEntries(entries));
            }
        };

        if (providers.length > 0) {
            fetchStates();
            const id = setInterval(fetchStates, 2000);

            return () => {
                mounted = false;
                clearInterval(id);
            };
        }
    }, [providers]);
    //@ts-ignore
    return <WinDropdown items={providers.map((e) => ({ name: t(`providerName.${e.name}`), icon: states[e.name] == ProviderState.Ready ? <CheckmarkCircleFilled /> : <DismissCircleFilled /> }))} defaultValue={currentProvider} onValueChange={(index) => {
        setCurrentProvider(index);
        const name = providers[index].name;
        localStorage.setItem(PROVIDER_STORAGE_KEY, name);
    }}></WinDropdown>
}
function Category({ option, selected, setSelected, offical }: { option: string, selected?: boolean, setSelected: (option: string) => void, offical: boolean }) {
    const deviceMap = useDeviceMap();
    const { t } = useI18n();
    return <div
        onClick={(e) => { e.stopPropagation(); setSelected(option) }}
        style={{ pointerEvents: "fill", padding: 10, borderRadius: 8, backgroundColor: `color-mix(in srgb, ${selected ? tokens.colorBrandBackgroundSelected : tokens.colorNeutralCardBackground}, transparent 10%)` }}>
        {offical ? Object.values(deviceMap || {}).find(device => device.codename === option)?.name ?? t(`resourceCategory.${option}`) : option}
    </div>
}
