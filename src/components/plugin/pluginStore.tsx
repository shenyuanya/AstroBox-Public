import { useI18n } from "@/i18n";
import usePluginList from "@/hooks/usePluginList";
import logger from "@/log/logger";
import { PluginManifest } from "@/plugin/types";
import { providerManager } from "@/pluginstore/manager";
import { Body1, Spinner } from "@fluentui/react-components";
import { useEffect, useMemo, useState } from "react";
import AutoSizer from "react-virtualized-auto-sizer";
import { FixedSizeList, ListChildComponentProps } from "react-window";
import InfiniteLoader from "react-window-infinite-loader";
import PluginCard from "@/components/plugincard/PluginCard";

interface PluginStoreProps {
    /**
     * Search text used to filter plugins on server side
     */
    search?: string;
    /**
     * Called when a plugin is clicked
     */
    onSelect?: (plugin: PluginManifest) => void;
}

// Item height inside virtual list
const ITEM_HEIGHT = 80;
const PAGE_SIZE = 20;
const PROVIDER_STORAGE_KEY = "pluginstore_provider";

export default function PluginStore({ search, onSelect }: PluginStoreProps) {
    const { t } = useI18n();
    // 1. 从 providerManager 获取 providers 列表和整体加载状态
    const { providers, loading: providersLoading } = providerManager.useProviders();

    const [currentProvider, setCurrentProvider] = useState<number>(0);
    const [hasMore, setHasMore] = useState(true);

    // 2. 只有在 providers 加载完成后才确定 providerName
    const providerName = useMemo(() => {
        if (providersLoading || providers.length === 0) {
            return null; // 正在加载或没有 provider 时，名称为 null
        }
        return providers[currentProvider]?.name;
    }, [providers, currentProvider, providersLoading]);

    const {
        data,
        setSize,
        size,
        isValidating,
    } = usePluginList(
        //@ts-ignore
        providerName,
        search,
        PAGE_SIZE
    );

    const plugins = useMemo(() => data?.flat() ?? [], [data]);

    // 检查是否还有更多数据
    useEffect(() => {
        if (data) {
            const lastPage = data[data.length - 1];
            if (lastPage === undefined || lastPage.length < PAGE_SIZE) {
                setHasMore(false);
            } else {
                setHasMore(true); // 如果加载了新页面，可能又有更多数据了
            }
        }
    }, [data]);

    // 当 provider 列表加载完成后，从 localStorage 初始化
    useEffect(() => {
        if (!providersLoading && providers.length > 0) {
            const stored = localStorage.getItem(PROVIDER_STORAGE_KEY);
            let idx = 0;
            if (stored) {
                const i = providers.findIndex(p => p.name === stored);
                if (i !== -1) idx = i;
            }
            setCurrentProvider(idx);
            logger.info(`current provider set to: ${providers[idx]?.name}`);
        }
    }, [providers, providersLoading]);

    // 切换 provider 或 search 时重置状态
    useEffect(() => {
        setHasMore(true);
    }, [currentProvider, search]);

    const itemCount = hasMore ? plugins.length + 1 : plugins.length;
    const isItemLoaded = (index: number) => !hasMore || index < plugins.length;
    const loadMoreItems = isValidating ? () => { } : () => setSize(size + 1);

    const Row = ({ index, style }: ListChildComponentProps) => {
        style = { ...style, width: "calc(100% - 20px)", overflow: "visible" };
        if (!isItemLoaded(index)) {
            return (
                <div style={{ ...style, display: 'flex', alignItems: 'center', justifyContent: 'center' }}>
                    {hasMore ? (
                        <Spinner size="tiny" />
                    ) : (
                        <div style={{ textAlign: 'center', padding: 16 }}>
                            <Body1>{t('plugin.storeEnd')}</Body1>
                        </div>
                    )}
                </div>
            );
        }
        const plugin = plugins[index];
        return (
            <div style={style}>
                <PluginCard plugin={plugin} onClick={() => onSelect?.(plugin)} />
            </div>
        );
    };

    // 4. 在 providers 列表加载时，显示顶层加载状态
    if (providersLoading) {
        return (
            <div style={{ padding: '20px', display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
                <Spinner size="tiny" label={t('plugin.loading')} />
            </div>
        );
    }

    if (providers.length === 0) {
        return (
            <div style={{ padding: '20px', display: 'flex', justifyContent: 'center', alignItems: 'center', height: '100%' }}>
                <Body1>{t('plugin.noPluginsInstalled')}</Body1> {/* 可以换成更合适的文案，比如 "没有可用的插件市场" */}
            </div>
        );
    }

    return (
        <div style={{ display: "flex", flexDirection: "column", flex: 1 }}>
            <div style={{ flex: 1 }}>
                <AutoSizer>
                    {({ height, width }) => (
                        <InfiniteLoader
                            isItemLoaded={isItemLoaded}
                            loadMoreItems={loadMoreItems}
                            itemCount={itemCount}
                        >
                            {({ onItemsRendered, ref }) =>
                                <FixedSizeList
                                    height={height}
                                    width={width}
                                    itemCount={itemCount}
                                    itemSize={ITEM_HEIGHT}
                                    onItemsRendered={onItemsRendered}
                                    ref={ref}
                                    style={{ overflow: "visible" }}
                                >
                                    {Row}
                                </FixedSizeList>}
                        </InfiniteLoader>
                    )}
                </AutoSizer>
            </div>
        </div>
    );
}