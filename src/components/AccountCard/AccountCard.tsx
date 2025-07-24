import { Avatar, Caption1, Card, CardHeader, Label } from "@fluentui/react-components";
import { ChevronRightRegular, OpenRegular } from "@fluentui/react-icons";
import styles from "./AccountCard.module.css";

type CardButtonProps = {
    avatar: string;
    content: string;
    onClick?: any;
    secondaryContent?: string;
    className?: string;
    disabled?: boolean;
    opener?: boolean;
};

export default function AccountCard({ avatar, content, onClick, secondaryContent, className, disabled, opener }: CardButtonProps) {
    const AvatarName = content;

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
                image={<Avatar name={AvatarName} image={{src:avatar}} color="colorful" style={{ marginLeft: "-2px", marginRight: "-2px" }}></Avatar>}

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
