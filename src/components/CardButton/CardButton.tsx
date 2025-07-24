import { Caption1, Card, CardHeader, Label } from "@fluentui/react-components";
import { ChevronRightRegular, OpenRegular } from "@fluentui/react-icons";
import styles from "./CardButton.module.css";

type CardButtonProps = {
    icon: React.ComponentType<any>;
    content: string;
    onClick?: any;
    secondaryContent?: string;
    className?: string;
    disabled?: boolean;
    opener?: boolean;
};

export default function CardButton({ icon, content, onClick, secondaryContent, className, disabled, opener }: CardButtonProps) {
    const IconComponent = icon;

    return (
        <Card
            className={`${styles.cardButton} card ${className}`}
            onClick={disabled ? undefined : onClick}
            aria-disabled={disabled}
        >
            <CardHeader
                style={{
                    opacity: disabled ? 0.5 : 1
                }}
                image={<IconComponent fontSize={28} appearance="inverted" size="small"  />}
                header={<Label weight="regular" className={styles.body}>{content}</Label>}
                description={<Caption1 className={styles.caption}>{secondaryContent}</Caption1>}
                // action={opener ? <OpenRegular fontSize={20} /> : <ChevronRightRegular fontSize={20} />}
                action={{
                    children: opener ? <OpenRegular fontSize={20} /> : <ChevronRightRegular fontSize={20} />,
                    className: styles.actionSlot,
                }}
            />
        </Card>
    );
}
