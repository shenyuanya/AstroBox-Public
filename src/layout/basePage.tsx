import { useAnimatedRouter } from '@/hooks/useAnimatedRouter';
import { isNav } from '@/router/nav';
import { Button } from "@fluentui/react-components";
import { ArrowLeft24Filled } from '@fluentui/react-icons';
import { AnimatePresence, motion } from 'framer-motion';
import { type ReactNode } from 'react';
import styles from './basePage.module.css';

interface BasePageProps {
  children?: ReactNode;
  title?: string;
  arrowColor?: string;
  externalTitle?: ReactNode;
  className?: string;
  action?: ReactNode;
  [key: string]: any;
}

export default function BasePage({ children, title, externalTitle, className, arrowColor, action, ...args }: BasePageProps) {
  const router = useAnimatedRouter();
  const ishome = isNav(router.pathname);

  const onclick = () => {
    if (ishome) return;
    router.back();
  }

  return (
    <div className={`${styles.container} ${className || ''}`} {...args}>
      <header className={styles.header}>
        <Button
          onClick={onclick}
          appearance="transparent"
          size="large"
          className={styles.backBtn}
          style={{ padding: 0, minWidth: "unset", cursor: "default", ["--arrowColor" as string]: arrowColor }}
        >
          <AnimatePresence>
            {!ishome && (
              <motion.div
                initial={{ opacity: 0, x: -10, width: 0 }}
                animate={{ opacity: 1, x: 0, width: 32 }}
                exit={{ opacity: 0, x: -10, width: 0 }}
                transition={{ duration: 0.3, delay: 0.1 }}
              >
                  <ArrowLeft24Filled className={styles.arrowIcon} />
              </motion.div>
            )}
          </AnimatePresence>

          <div className={styles.titleRow}>
            <h1 className={styles.title}>{title}</h1>
            {externalTitle}
          </div>
        </Button>
        {action}
      </header>
      {children}
    </div>
  );
}