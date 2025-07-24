import ModifyWatchfaceIDDialog from "@/components/ModifyWatchfaceIDDialog/ModifyWatchfaceIDDialog";
import { ResourceType } from "@/device/install";
import { TaskItem } from "@/taskqueue/tasklist";
import {
  Button,
  CardHeader,
  Field,
  ProgressBar,
  Text,
  makeStyles,
  tokens
} from "@fluentui/react-components";
import { DismissRegular } from "@fluentui/react-icons";
import styles from "./queuecard.module.css";

const useStyles = makeStyles({
  iconContainer: {
    width: "32px",
    height: "32px",
    display: "flex",
    alignItems: "center",
    justifyContent: "center",
  },
  progressContainer: {
    marginTop: tokens.spacingVerticalM,
    paddingBlock: 0,
  },
  progressLabel: {
    display: "flex",
    justifyContent: "space-between",
    alignItems: "center",
    marginBottom: tokens.spacingVerticalXS,
  },
});

interface QueueCardProps {
  onCancel?: () => void;
  isMobile?: boolean;
  item: TaskItem;
}

export default function QueueCard({
  item,
  onCancel,
  isMobile,
}: QueueCardProps) {
  const classes = useStyles();
  const showProgress = item.payload?.status != "pending";
  const IconComponent = item.icon
  const title = item.name || item.id;
  const description = item.description || item.id;

  const progressMap = {
    "pending": "none" as const,
    "running": "none" as const,
    "success": "success" as const,
    "error": "error" as const,
    "undefined": "none" as const,
  }
  const progressState = progressMap[item.payload?.status || "undefined"];

  return (
    <div className={styles.card}>
      <CardHeader
        style={{
          padding: 4
        }}
        image={
          <div className={classes.iconContainer}>
            <IconComponent style={{ fontSize: "28px", color: tokens.colorNeutralForeground2 }} />
          </div>
        }
        header={<Text weight="semibold" size={300} style={{ maxWidth: 200, overflow: "hidden", textOverflow: "ellipsis", textWrap: "nowrap" }}>{title}</Text>}
        description={
          <Text size={200}
            className={styles.description}
            style={{
              overflow: "hidden",
              overflowY: "auto",
              maxWidth: "200px",
              maxHeight: "5ch",
            }}
          >
            {description}
          </Text>
        }
        action={
          (!isMobile && onCancel) ? (
            <div style={{ display: 'flex', gap: '4px' }}>
              {item.type === ResourceType.WatchFace && (
                <ModifyWatchfaceIDDialog item={item} />
              )}
              <Button appearance="subtle" icon={<DismissRegular />} size="small" onClick={onCancel}>
              </Button>
            </div>
          ) : null
        }
      />

      {showProgress && (
        <div className={classes.progressContainer}>
          <Field validationMessage={((item.payload?.progress ?? 0) * 100).toFixed(2) + "% " + (item.payload?.progressDesc ?? "")} validationState={progressState}>
            <ProgressBar value={item.payload?.progress} thickness="medium" shape="rounded" />
          </Field>
        </div>
      )}
    </div>
  );
}