import { useEffect, useState } from "react";

export default function useIsMobile(breakPoint = 768) {
    const [isMobile, setIsMobile] = useState(
        typeof window !== "undefined" ? window.innerWidth <= breakPoint : false,
    );

    useEffect(() => {
        const mq = window.matchMedia(`(max-width:${breakPoint}px)`);
        const handler = (e: MediaQueryListEvent | MediaQueryList) =>
            setIsMobile((e as MediaQueryList).matches);
        handler(mq);
        mq.addEventListener?.("change", handler);
        // safari 旧版
        // @ts-ignore
        mq.addListener?.(handler);
        return () => {
            mq.removeEventListener?.("change", handler);
            // @ts-ignore
            mq.removeListener?.(handler);
        };
    }, [breakPoint]);

    return isMobile;
}