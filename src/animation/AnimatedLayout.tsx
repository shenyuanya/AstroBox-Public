import { AnimatePresence, motion } from "framer-motion";
import { usePathname } from "next/navigation";
import { useEffect, useRef, useState } from "react";
import { useNavDirection } from "./NavDirectionContext";
import { NAV_EVENT } from "./navigationEvent";

export default function AnimatedLayout({ children }: { children: React.ReactNode }) {
  const pathname = usePathname();
  const { direction, setDirection } = useNavDirection();
  const [visible, setVisible] = useState(true);
  const callbackRef = useRef<(() => Promise<void>) | undefined>();

  useEffect(() => {
    const pop = () => setDirection("back");
    window.addEventListener("popstate", pop);
    return () => window.removeEventListener("popstate", pop);
  }, [setDirection]);

  useEffect(() => {
    const handler = (e: Event) => {
      const ce = e as CustomEvent<() => Promise<void>>;
      callbackRef.current = ce.detail;
      setVisible(false);
    };
    window.addEventListener(NAV_EVENT, handler);
    return () => window.removeEventListener(NAV_EVENT, handler);
  }, []);

  // 自定义 easing 曲线
  const easing = {
    // 进场带轻微弹性（overshoot）
    overshoot: [0.175, 0.885, 0.32, 1.275] as [number, number, number, number],
    // 退场丝滑收尾
    smooth: [0.24, 0.99, 0.44, 0.99] as [number, number, number, number],
  };

  const variants = {
    forward: {
      initial: { opacity: 0, x: 80 },
      animate: {
        opacity: 1,
        x: 0,
        transition: {
          duration: 0.28,        // 稍微加速
          ease: easing.overshoot,
        },
      },
      exit: {
        opacity: 0,
        x: -80,
        transition: {
          duration: 0.2,
          ease: easing.smooth,
        },
      },
    },
    back: {
      initial: { opacity: 0, x: -80 },
      animate: {
        opacity: 1,
        x: 0,
        transition: {
          duration: 0.28,
          ease: easing.overshoot,
        },
      },
      exit: {
        opacity: 0,
        x: 80,
        transition: {
          duration: 0.2,
          ease: easing.smooth,
        },
      },
    },
    none: {
      initial: { opacity: 1, x: 0 },
      animate: { opacity: 1, x: 0 },
      exit: { opacity: 1, x: 0 },
    },
  };

  return (
    <AnimatePresence
      mode="wait"
      initial={false}
      onExitComplete={async () => {
        if (callbackRef.current) {
          try {
            await callbackRef.current();
          } finally {
            callbackRef.current = undefined;
            setVisible(true);
          }
        } else {
          setVisible(true);
        }
      }}
    >
      {visible && (
        <motion.div
          key={pathname}
          initial={variants[direction].initial}
          animate={variants[direction].animate}
          exit={variants[direction].exit}
          onAnimationComplete={() => {
            if (direction === 'back') {
              setDirection('forward');
            }
          }}
          style={{
            height: "100%",
            display: "flex",
            flexDirection: "column",
            flexGrow: 1,
            overflow: "hidden",
          }}
        >
          {children}
        </motion.div>
      )}
    </AnimatePresence>
  );
}