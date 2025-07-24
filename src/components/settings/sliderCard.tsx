import { Text, makeStyles } from "@fluentui/react-components";
import { useEffect, useState } from "react";
import AppleLikeSlider from "./appleLikeSlider";
import SettingsCard from "./settingsCard";

const useStyles = makeStyles({
    root: {
        display: "flex",
        flexDirection: "row",
        alignItems: "center",
        '@media (max-width: 512px)': {
            flexDirection: "row-reverse",
            justifyContent: "start"
        }
    },
    action: {
        '@media (max-width: 512px)': {
            gridColumnStart: 2,
            marginLeft: "-10px",
            marginRight: "2px",
            marginBottom: "-4px",
            marginTop: "4px",
        },
    },
    actionText: {
        minWidth: "3em", textAlign: "right",
        '@media (max-width: 512px)': {
            minWidth: "unset", textAlign: "left"
        },
    },
});

export default function SwitchCard({ max, min, value, step, defaultValue, onChange, unit, ...props }: { value?: number, unit?: string, min: number, max: number, step?: number, title?: string, desc?: string, defaultValue?: number, onChange?: (item: number) => void, Icon?: React.ComponentType<any> }) {
    const [curValue, setValue] = useState(value ?? defaultValue ?? min);
    const styles = useStyles();
    useEffect(() => { value && setValue(value) }, [value])
    return <SettingsCard {...props} actionProps={{
        className: styles.action
    }}
    // classStyle={styles.action}
    >
        <div className={styles.root}>
            <Text className={styles.actionText}>{curValue}{unit}</Text>
            <AppleLikeSlider min={min} max={max} step={step} value={curValue} onChange={(e, data) => {
                setValue(data.value);
                onChange?.(data.value);
            }} />
        </div>
    </SettingsCard>
}