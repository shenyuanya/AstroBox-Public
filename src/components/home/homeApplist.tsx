import {providerManager} from '@/community/manager';
import CommunityWatchfaceCard from '@/components/home/CommunityWatchfaceCard';
import {useAnimatedRouter} from '@/hooks/useAnimatedRouter';
import useIsMobile from '@/hooks/useIsMobile';
import BasePage from '@/layout/basePage';
import { useI18n } from '@/i18n';
import {Item} from '@/types/ResManifestV1';
import {Body1, Button, Portal, Skeleton, SkeletonItem, Spinner} from '@fluentui/react-components';
import {AnimatePresence, motion} from 'framer-motion';
import {useCallback, useEffect, useMemo, useRef, useState} from 'react';
import WinDropdown from '../settings/winDropdown';
import styles from './homeApplist.module.css';
import useAppList from "@/hooks/useAppList";


const PROVIDER_STORAGE_KEY = "community_provider";

export default function AppList({search}: { search?: string }) {
  const router = useAnimatedRouter();
  const { t } = useI18n();
  const [currentProvider, setCurrentProvider] = useState<number>();

  const { providers, loading: providersLoading, errors: providersErrors, lastUpdated } = providerManager.useProviders();
  const isMobile = useIsMobile();
  const [hasMore, setHasMore] = useState(true);
  const [animationRef, setAnimationRef] = useState<any>(null);

  const fuckWebkit = useMemo(() => localStorage.getItem("fkWebkit") === "true", []);

  const observer = useRef<IntersectionObserver | null>(null);
  const limit = 10;

  const {
    data,
    mutate,
    setSize,
    size,
    isValidating,
    error
  } = useAppList(providers[currentProvider ?? 114514]?.name, search, limit)

  const handleUserRefresh = useCallback(() => {
    providerManager.refreshAll();
  }, []);

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
    setHasMore(true);
    mutate()
  }, [providers, lastUpdated, search]);

  const lastItemRef =
    (node: HTMLDivElement | null) => {
      if (isValidating) return;
      if (observer.current) observer.current.disconnect();

      observer.current = new IntersectionObserver(entries => {
        if (entries[0].isIntersecting && hasMore) {
          setSize(size + 1).then((entries) => {
            if ((entries?.toReversed()?.[0]?.length ?? 0) < limit) setHasMore(false)
          })
        }
      });

      if (node) observer.current.observe(node);
    }

  const onItemClick = (item: Item, e: React.MouseEvent) => {
    e.stopPropagation();
    setAnimationRef({
      width: e.currentTarget?.clientWidth,
      height: e.currentTarget?.clientHeight,
      x: e.currentTarget?.getBoundingClientRect().x,
      y: e.currentTarget?.getBoundingClientRect().y,
      styles: { transition: "none", borderRadius: "var(--borderRadiusXLarge)" },
      item
    });
    requestAnimationFrame(() => {
      setAnimationRef({
        item
      });
    })
  };
  const renderContent = () => {
    const totalLoading = providersLoading || (isValidating && !(data?.length !== 0));
    if (totalLoading) {
      return (
        <div className={styles.list}>
          <Skeleton aria-label="Loading Content" className={styles.list} appearance="translucent">
            {Array(6).fill(1).map((_, i) => <SkeletonItem key={i} style={{ height: isMobile ? 195 : 246, borderRadius: "var(--borderRadiusXLarge)" }} />)}
          </Skeleton>
        </div>
      );
    }

    if (error) {
      return (
        <div className={styles.end}>
          <p>{error.toString()}</p>
          <Button onClick={handleUserRefresh} disabled={providersLoading} appearance="outline">
            {providersLoading ? t('home.loading') : t('home.retry')}
          </Button>
        </div>
      );
    }

    if (data?.length == 0) {
      return <div className={styles.end}>{t('home.none')}</div>;
    }

    return (
      <>
        <div className={styles.list}>
          <AnimatePresence>
            {data?.map((page, pindex) => page.map((item, index) => (
              <motion.div
                  initial={{opacity: 0, transform: fuckWebkit ? "" : "translateY(20px)"}}
                  animate={{opacity: 1, transform: fuckWebkit ? "" : "translateY(0)"}}
                transition={{duration: 0.3, delay: index * 0.05}}
                key={`${item.name}-${(pindex + 1) * (index + 1)}`}
                className={styles.listItem}
              >
              <CommunityWatchfaceCard item={item} onClick={(e) => onItemClick(item, e)} />
            </motion.div>
            )))}
          </AnimatePresence>

        </div>
        {hasMore && <div className={styles.loading} ref={lastItemRef}><Spinner size="tiny" /><Body1 style={{color:"var(--colorNeutralForeground1)"}}>{t('home.loadingMore')}</Body1></div>}
        {!hasMore && !isValidating && <div className={styles.end}>{t('home.bottom')}</div>}
      </>
    );
  };

  return (
    <div className={styles.container}>
      <div style={{ display: "flex", flexDirection: "row", justifyContent: "space-between" }}>
        <h2 className={styles.header}>{t('home.appList')}</h2>
        {/* @ts-ignore */}
        <WinDropdown items={providers.map((e) => t(`providerName.${e.name}`))} defaultValue={currentProvider} appearance="transparent" onValueChange={(index) => {
          setCurrentProvider(index);
          const name = providers[index].name;
          localStorage.setItem(PROVIDER_STORAGE_KEY, name);
          mutate()
          setHasMore(true);
        }}></WinDropdown>
      </div>
      {renderContent()}
      <Portal mountNode={{ className: "portal" }}>
        <div
          className={styles.animation}
          style={{
            width: animationRef?.width,
            height: animationRef?.height,
            top: animationRef?.y,
            left: animationRef?.x,
            ...animationRef?.styles,
            display: animationRef ? "flex" : "none",
          }}
          onTransitionEnd={() => {
            if (currentProvider === undefined) return setAnimationRef(undefined);
            const name = providers[currentProvider].name !== "bandbbs" ? animationRef?.item.name : animationRef?.item._bandbbs_ext_resource_id;
            router.pushWithoutAnimation({
              pathname: '/community/product-info/product-info',
              query: { name: name, provider: providers[currentProvider].name },
            });
          }}
        >
          <BasePage title={t('product.detail')}>
          </BasePage>
        </div>
      </Portal>
    </div>
  );
}