import {
  Button,
  type SelectTabData,
  type SelectTabEvent,
  Tab,
  TabList,
  type TabValue,
  Text,
} from '@fluentui/react-components';
import { Dismiss24Regular } from '@fluentui/react-icons';
import * as React from 'react';
import CommandPoolExplorer from './CommandPoolExplorer';
import styles from './DebugWindow.module.css';
import PluginStoreTest from './PluginStoreTest';

export const DebugWindow: React.FC = () => {
  const [open, setOpen] = React.useState(false);
  const [selected, setSelected] = React.useState<TabValue>('commandpool');

  const handleTabSelect = (_e: SelectTabEvent, data: SelectTabData) =>
    setSelected(data.value);

  return (
    <>
      <img
        className={styles['floating-debug-button']}
        onClick={() => setOpen(true)}
        aria-label="Open debug window"
        src="/debuggm.png"
      >
      </img>

      {open && (
        <div
          className={styles['debug-window']}
          role="dialog"
          aria-modal={false}
          aria-label="Debug window"
        >
          <div className={styles['debug-window-header']}>
            <Text as="h2" weight="semibold">
              Debug Window
            </Text>
            <Button
              appearance="subtle"
              aria-label="Close debug window"
              icon={<Dismiss24Regular />}
              onClick={() => setOpen(false)}
            />
          </div>

          <TabList
            selectedValue={selected}
            onTabSelect={handleTabSelect}
            className={styles['debug-tablist']}
            size="small"
          >
            <Tab value="commandpool">CommandPool</Tab>
            <Tab value="btstack">BtStack</Tab>
            <Tab value="pst">PluginStore</Tab>
          </TabList>

          <div className={styles['debug-tab-panel']}>
            {selected === 'commandpool' && (
              <CommandPoolExplorer></CommandPoolExplorer>
            )}
            {selected === 'pst' && (
              <PluginStoreTest></PluginStoreTest>
            )}
          </div>
        </div>
      )}
    </>
  );
};

export default DebugWindow;