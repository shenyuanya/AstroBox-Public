import { Slider, SliderProps } from "@fluentui/react-components";
import { useMemo, useRef, useState } from "react";

export default function AppleLikeSlider(props: SliderProps) {
    const [down, setDown] = useState(false);
    const sliderRef = useRef<HTMLInputElement>(null);
    const [style, setStyle] = useState(props.style);
    const originRect = useMemo(() => {
        return sliderRef.current?.getBoundingClientRect();
    },[down])
    return <Slider {...props} style={style} ref={sliderRef} onPointerMove={(e) => {
        if (!down) return;
        if (!originRect) return;
        const overLeft = Math.max(Math.min(0,e.clientX - originRect?.left),-20);
        const overRight = Math.min(Math.max(0,e.clientX - originRect?.right),20);
        let over = Math.abs(overLeft) < Math.abs(overRight) ? overRight : overLeft;
        setStyle({
            ...style,
            transform: `matrix(${1 + Math.abs(over) / originRect.width}, 0, 0, 1, ${reduceFactor(over, originRect.width * 2)}, 0)`,
        })
    }} onPointerDown={() => { setDown(true) }} onPointerUp={() => {
        setDown(false)
        setStyle({
            ...style,
            transform: `matrix(1, 0, 0, 1, 0, 0)`,
            transition: "transform .2s var(--bouncy)",
        })
    }} />;

}
function reduceFactor(num: number, sorce: number = 1) {
    return (1 - Math.abs(num) / sorce) * num * .5
}