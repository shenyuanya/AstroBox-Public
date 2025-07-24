import { useI18n } from "@/i18n";
import { Button, Portal, Tab, TabList } from "@fluentui/react-components";
import { ChevronDownRegular } from "@fluentui/react-icons";
import { useEffect, useRef, useState } from "react";
import AppleButtonWrapper from "../appleButtonWapper/appleButtonWapper";
import styles from "./dropdown.module.css";

interface DropDownProps extends Parameters<typeof Button>{
    items: string[];
    defaultValue?: number;
    onValueChange?: (value: number) => void;
}

export default function WinDropdown({ items, defaultValue = 0, onValueChange, ...props }: DropDownProps) {
    const { t } = useI18n();
    const [curValue, setValue] = useState(defaultValue);
    const [open, setOpen] = useState(false);
    const [dropdownPosition, setDropdownPosition] = useState({
        left: 0,
        top: 0,
        width: 0,
        marginTop:0
    });
    const currentRef = useRef<HTMLButtonElement>(null);
    const listRef = useRef<HTMLDivElement>(null);
    const btnRef = useRef<HTMLButtonElement>(null);

    useEffect(() => {
        if (!open) return;

        const handleClickOutside = (e: MouseEvent) => {
            const target = e.target as Node;
            if (!btnRef.current?.contains(target) && !listRef.current?.contains(target)) {
                setOpen(false);
            }
        };

        document.addEventListener('click', handleClickOutside);
        return () => document.removeEventListener('click', handleClickOutside);
    }, [open]);

    useEffect(() => {setValue(defaultValue); }, [defaultValue]);

    const updateDropdownPosition = () => {
        if (btnRef.current) {
            const rect = btnRef.current.getBoundingClientRect();
            setDropdownPosition({
                left: rect.left,
                top: rect.top,
                width: rect.width + 10,
                marginTop: - (currentRef.current?.getBoundingClientRect()?.top ?? 0) + (listRef.current?.getBoundingClientRect()?.top ?? 0),
            });
        }
    };

    const handleClick = () => {
        setOpen((prevOpen) => !prevOpen);
    };

    useEffect(() => {
        updateDropdownPosition();
        window.addEventListener('resize', updateDropdownPosition);
        window.addEventListener('scroll', updateDropdownPosition, true); // Use capture phase to ensure it runs before dropdown's own scroll handling
        return () => {
            window.removeEventListener('resize', updateDropdownPosition);
            window.removeEventListener('scroll', updateDropdownPosition, true);
        };
    }, [curValue, open]);
    return (<>
        <AppleButtonWrapper padding={5}>
            <Button
                appearance="transparent"
                {...props}
                icon={<ChevronDownRegular />} iconPosition="after"
                style={{ justifyContent: "space-between", background: "#ffffff11", border: "var(--strokeWidthThin) solid #00000011" }}
                ref={btnRef}
                onClick={(e) => {
                    e.stopPropagation();
                    handleClick();
                }}>{t(items[curValue])}</Button>
        </AppleButtonWrapper>

        <Portal>
            <TabList
                vertical
                ref={listRef}
                onClick={(e) => e.stopPropagation()}
                onTabSelect={(e, { value }: { value: any }) => {
                    setValue(value);
                    onValueChange?.(value);
                    setOpen(false);
                }}
                onBlur={() => {
                    setOpen(false);
                }}
                style={{
                    opacity: open ? 1 : 0,
                    pointerEvents: open ? "auto" : "none",
                    boxSizing: "border-box",
                    position: "fixed", // Changed to fixed
                    background: "color-mix(in srgb,var(--colorNeutralBackground1),transparent 10%)",
                    backdropFilter: "blur(10px)",
                    borderRadius: "var(--borderRadiusXLarge)",
                    border: "var(--strokeWidthThin) solid var(--colorNeutralStroke3)",
                    transition: "margin-top 0.1s ease-in,opacity 0.1s ease-in",
                    boxShadow: "rgba(0, 0, 0, 0.24) 0px 0px 2px, rgba(0, 0, 0, 0.28) 0px 4px 8px",
                    gap: "1px",
                    padding: "5px",
                    margin: "0 -5px",
                    ...dropdownPosition,
                }}
                selectedValue={curValue}
            >
                {items.map((item, index) => (
                    <Tab className={styles.tab} ref={(() => { if (index === curValue) return currentRef; })()} key={index} value={index}
                        style={{
                            background: index === curValue ? "color-mix(in srgb, var(--colorNeutralBackground2Hover), transparent 10%)" : "",
                        }}
                    >{t(item)}</Tab>
                ))}
            </TabList>
        </Portal>
    </>)
}