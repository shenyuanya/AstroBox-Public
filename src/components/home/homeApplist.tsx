import { providerManager } from '@/community/manager';
import CommunityWatchfaceCard from '@/components/home/CommunityWatchfaceCard';
import { useAnimatedRouter } from '@/hooks/useAnimatedRouter';
import useAppList from "@/hooks/useAppList";
import useIsMobile from '@/hooks/useIsMobile';
import { useI18n } from '@/i18n';
import BasePage from '@/layout/basePage';
import { Provider, ProviderState } from '@/plugin/types';
import { Item } from '@/types/ResManifestV1';
import { Body1, Button, Portal, Skeleton, SkeletonItem, Spinner } from '@fluentui/react-components';
import { AnimatePresence, motion } from 'framer-motion';
import { useCallback, useEffect, useMemo, useRef, useState } from 'react';
import useSWR from 'swr';
import styles from './homeApplist.module.css';

export default function AppList({ search, provider, categories }: { provider?: Provider, categories?: string[], search?: string }) {
  const router = useAnimatedRouter();
  const { t } = useI18n();
  const { errors: providersErrors } = providerManager.useProviders();
  const isMobile = useIsMobile();
  const [hasMore, setHasMore] = useState(true);
  const [animationRef, setAnimationRef] = useState<any>(null);

  const fuckWebkit = useMemo(() => localStorage.getItem("fkWebkit") === "true", []);

  const observer = useRef<IntersectionObserver | null>(null);
  const limit = 10;

  const { data: providerState } = useSWR(provider?.name, () => provider?.getState(), { refreshInterval: 100 });

  const {
    data,
    mutate,
    setSize,
    size,
    isValidating,
    isLoading,
    error
  } = useAppList(providerState === ProviderState.Ready ? provider?.name : undefined, search, categories, limit)

  const handleUserRefresh = useCallback(() => {
    providerManager.refreshAll();
  }, []);
  useEffect(() => {
    setHasMore(true);
  }, [provider?.name, search, categories])
  const lastItemRef =
    (node: HTMLDivElement | null) => {
      if (isValidating) return;
      if (observer.current) observer.current.disconnect();

      observer.current = new IntersectionObserver(entries => {
        if (entries[0].isIntersecting && hasMore && providerState === ProviderState.Ready) {
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
    const totalError = providersErrors.find((item) => item.name === provider?.name);
    if (providerState === ProviderState.Failed || error || totalError) {
      return (
        <div className={styles.end}>
          <p>{error?.toString() ?? totalError?.e?.toString()}</p>
          <Button onClick={handleUserRefresh} appearance="outline">
            {t('home.retry')}
          </Button>
        </div>
      );
    }

    if (isLoading || providerState === ProviderState.Updating) {
      return (
        <div className={styles.list}>
          <Skeleton aria-label="Loading Content" className={styles.list} appearance="translucent">
            {Array(6).fill(1).map((_, i) => <SkeletonItem key={i} style={{ height: isMobile ? 195 : 246, borderRadius: "var(--borderRadiusXLarge)" }} />)}
          </Skeleton>
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
            const name = provider?.name !== "bandbbs" ? animationRef?.item.name : animationRef?.item._bandbbs_ext_resource_id;
            router.pushWithoutAnimation({
              pathname: '/community/product-info/product-info',
              query: { name: name, provider: provider?.name },
            });
            setTimeout(() => {
              setAnimationRef(undefined);
            }, 300);
          }}
        >
          <BasePage title={t('product.detail')}>
          </BasePage>
        </div>
      </Portal>
    </div>
  );
}