import { Body1, Caption1, CardHeader, Slot, makeStyles, mergeClasses } from "@fluentui/react-components";

const useStyles = makeStyles({
    body: {
        "@media (max-width: 512px)": {
            fontSize: "15px",
            lineHeight: "1.4",
        },
    },
    caption: {
        opacity: 0.7,
        "@media (max-width: 512px)": {
            fontSize: "13px",
            lineHeight: "1.4",
        },
    },
});

export default function SettingsCard({
    title, desc, Icon, children,
    actionProps, // parent 可以传 { className, style }
}: {
    title?: string;
    desc?: string;
    Icon?: React.ComponentType<any>;
    children?: React.ReactNode;
    actionProps?: React.HTMLAttributes<HTMLDivElement>;
}) {
    const styles = useStyles();
    return <div className="card" style={{ borderRadius: "var(--borderRadiusXLarge)" }}>
        <CardHeader
            image={Icon && <Icon fontSize={28} />}
            header={<Body1 className={styles.body}>{title}</Body1>}
            description={<Caption1 className={styles.caption}>{desc}</Caption1>}
            // action={children}
            action={{
                className: mergeClasses(actionProps?.className),
                style: actionProps?.style,
                children,
            }}
            style={{ gridAutoColumns: "min-content 1fr" }}
        />
    </div>
}