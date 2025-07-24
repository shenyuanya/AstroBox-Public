import useIsMobile from "@/hooks/useIsMobile";
import { useEffect, useRef, useState } from "react";
import { useI18n } from "@/i18n";
import styles from "./HomeNav.module.css";
import NavItem from "./NavItem";

type NavItemDetail = {
    name: string;
    label: string;
    iconRegular: React.ComponentType<any>;
    iconFilled: React.ComponentType<any>;
};

type HomeNavProps = {
    navItems: NavItemDetail[];
    currentNavItem: string;
    onNavItemClick: (itemName: string) => void;
    className?: string;
};

export default function HomeNav({ navItems, currentNavItem, onNavItemClick, className }: HomeNavProps) {
    const navContainerClasses = [styles.nav, className || ""].join(" ").trim();
    const { t } = useI18n();
    const [indicatorStyle, setIndicatorStyle] = useState({ top: "0px",height:"22px" });
    const currentRef = useRef<HTMLDivElement | null>(null);
    const indicatorRef = useRef<HTMLDivElement>(null);
    const isMobile = useIsMobile();

    useEffect(() => {
        if (currentRef.current&&indicatorRef.current&&!isMobile) {
            const rect = currentRef.current.getBoundingClientRect();
            const indicatorRect = indicatorRef.current.getBoundingClientRect();
            if (indicatorRect.top === rect.top)return;
            if (indicatorRect.top > rect.top) {
                setIndicatorStyle({
                    top: `${rect.top}px`,
                    height: `${indicatorRect.top-rect.top}px`
                })
            } else {
                setIndicatorStyle({
                    top: `${indicatorRect.top}px`,
                    height: `${22 + rect.top - indicatorRect.top}px`
                })
            }
        }
    }, [currentNavItem,isMobile]);
    return (
        <nav className={navContainerClasses}>
            {navItems.map((item,index) => {
                const isActive = currentNavItem === item.name;
                const itemClasses = [styles["nav-item"], isActive ? styles.active : ""].join(" ").trim();

                const IconComponent = isActive ? item.iconFilled : item.iconRegular;

                return (
                    <NavItem
                        className={itemClasses}
                        key={item.name}
                        refcall={(el: HTMLDivElement | null) =>
                        {
                            if (isActive&&el!==null){
                                currentRef.current = el;
                            }
                        }
                        }
                        onClick={() => onNavItemClick(item.name)}
                        role="button"
                        tabIndex={0}
                        name={t(item.label)}
                        IconComponent={IconComponent}
                    />
                );
            })}
            <div className={styles["nav-tail"]} style={indicatorStyle} ref={indicatorRef} onTransitionEnd={()=>{
                setIndicatorStyle({
                    top: `${currentRef!.current!.getBoundingClientRect().top}px`,
                    height: `22px`
                })
            }}/>
        </nav>
    );
}