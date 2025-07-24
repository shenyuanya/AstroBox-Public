import { cloneElement, useRef, useState } from 'react';
import styles from './appleButtonWapper.module.css';


/** 苹果跟随鼠标按钮包装器
 *  Note: Please do not directly override the transition of child components!!!
 *  @param children child components
 *  @param padding The range of mouse follow
 *  @param transition Additional transition for child components
 */
export default function AppleButtonWrapper({ children,padding=10,transition }: { children: React.ReactElement<any>,padding?:number,transition?:string }) {
    const transition1 = "transform 500ms var(--bouncy)"
        transition?","+transition:""
    const [style, setStyle] = useState({ transform: "matrix(1, 0, 0, 1, 0, 0)",transition:transition1 })
    const overlay = useRef<any>(null)

    const handleMouse = (e: React.PointerEvent<HTMLDivElement>) => {
        if(e.pointerType !== "mouse")return;
        const rect = e.currentTarget.getBoundingClientRect()
        const computedX = e.clientX - rect.x - rect.width / 2;
        const computedY = e.clientY - rect.y - rect.height / 2;
        setStyle({ transform: `matrix(${1 + Math.abs(computedX / rect.width / 5.5)}, 0, 0, ${1 + Math.abs(computedY / rect.height/ 5.5)}, ${reduceFactor(computedX, rect.width-padding*2)}, ${reduceFactor(computedY, rect.height-padding*2)})`, transition:  "transform 0.05s ease-in-out"+transition?","+transition:""})
    }
    const handleLeave = () => {
        setStyle({ transform: "matrix(1, 0, 0, 1, 0, 0)", transition: transition1 })
    }
    return (
        <div className={styles.wrapper} style={{width:children.props?.style?.width,height:children.props?.style?.height}}>
            {cloneElement(children, { tabIndex: -1, style: { ...children.props.style, opacity: 0,pointerEvents:"none"}})}
            <div className={styles.overlay}
                style={{left: -padding, top: -padding,bottom: -padding,right:-padding}}
                ref={overlay}
                onPointerMove={handleMouse} onPointerEnter={handleMouse} onMouseLeave={handleLeave}>
                {cloneElement(children, {
                    style: { ...children.props.style, ...style ,position:"relative",margin:"auto"}
                })}
            </div>
        </div>
    );
}
function reduceFactor(num: number, sorce: number = 1) {
    return (1 - Math.abs(num) / sorce) * num * 0.2
}