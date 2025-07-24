import { Label } from "@fluentui/react-components";
import { listen, TauriEvent } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import styles from "./HomeNav.module.css";

export default function NavItem({ IconComponent,name ,refcall,...args }: {
    IconComponent: React.ComponentType<any>;
    name: string;
    refcall?: (el: HTMLDivElement | null) => void;
    [key: string]: any;
}) {
    const transition = "transform 500ms var(--bouncy)"
    const [transform, setTransform] = useState({ transform: "translate(0px,0px)", transition })
    const icon = useRef<any>(null)
    const [iconPosition, setIconPosition] = useState<DOMRect>()
    useEffect(() => {
        let unlisten: any = null;
        listen(TauriEvent.WINDOW_RESIZED, () => {
            if (icon.current) {
                setIconPosition(icon.current.getBoundingClientRect())
            }
        }).then((e) => unlisten = e);
        if (icon.current) {
            setIconPosition(icon.current.getBoundingClientRect())
        }
        return () => { unlisten?.(); }
    },[icon])
    const handleMouse = (e: React.PointerEvent<HTMLDivElement>) => {
        if (e.pointerType !== "mouse") return;
        const rect = iconPosition!
        const computedX = e.clientX - rect.x - rect.width / 2;
        const computedY = e.clientY - rect.y - rect.height / 2;
        setTransform({ transform: `translate(${reduceFactor(computedX)}px,${reduceFactor(computedY)}px)`, transition:"transform 50ms ease-in-out" })
    }
    return (
        <div
            ref={refcall}
            {...args}
            onPointerEnter={handleMouse}
            onPointerMove={handleMouse}
            onMouseLeave={() => setTransform({ transform: "translate(0px,0px)", transition })}
        >
            <IconComponent style={{ fontSize: "22px", ...transform }} className={styles["nav-icon"]} ref={icon} />
            <Label className={styles["nav-label"]}>
                {name}
            </Label>
        </div>
    )
}
function reduceFactor(num: number) {
    return (1-1/Math.abs(num))*num*0.2
}