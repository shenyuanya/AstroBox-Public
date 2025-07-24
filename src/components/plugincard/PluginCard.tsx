import { Subtitle2, Body1, CardHeader } from "@fluentui/react-components";
import { CheckboxCheckedFilled, CheckboxUncheckedFilled, PuzzlePiece48Regular } from "@fluentui/react-icons";
import Image from "next/image";
import styles from "./pluginCard.module.css";

export default function PluginCard({ plugin, local, ...props }: any) {
    return (
        <div className={styles.card} {...props}>

            <CardHeader
                header={<Subtitle2>{plugin.name}</Subtitle2>}
                description={<Body1 style={{ opacity: 0.75 }}>{plugin.description}</Body1>}
                image={plugin.icon ? <Image src={plugin.icon} alt={plugin.name} width={36} height={36} style={{
                    borderRadius:
                        "50%", border: "1px solid #00000013"
                }} /> : <PuzzlePiece48Regular style={{ height: "30px", width: "30px" }} />}

                style={{ width: "100%" }}
                action={{
                    children: local && (!plugin.disabled ? <CheckboxCheckedFilled style={{ height: "30px", width: "30px" }} /> : <CheckboxUncheckedFilled style={{ height: "30px", width: "30px" }} />),
                    className: styles.actionSlot
                }}
            />
        </div>
    )
}