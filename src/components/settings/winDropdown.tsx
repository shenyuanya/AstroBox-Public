import { useI18n } from "@/i18n";
import { Button, Portal, Slot, Tab, TabList } from "@fluentui/react-components";
import { ChevronDownRegular } from "@fluentui/react-icons";
import { HtmlHTMLAttributes, useEffect, useLayoutEffect, useRef, useState } from "react";
import AppleButtonWrapper from "../appleButtonWapper/appleButtonWapper";
import styles from "./dropdown.module.css";

interface DropDownProps extends HtmlHTMLAttributes<HTMLButtonElement> {
    items: (string | { name: string, icon: Slot<'span'> })[];
    defaultValue?: number;
    onValueChange?: (value: number) => void;
    maxDropdownWidth?: number;
}

export default function WinDropdown({
    items,
    defaultValue = 0,
    onValueChange,
    maxDropdownWidth = 320,
    ...props
}: DropDownProps) {
    const { t } = useI18n();
    const [curValue, setValue] = useState(defaultValue);
    const [open, setOpen] = useState(false);
    const [dropdownPosition, setDropdownPosition] = useState({
        left: 0,
        top: 0,
        marginTop: 0,
    });

    const btnRef = useRef<HTMLButtonElement>(null);
    const currentRef = useRef<HTMLButtonElement>(null);
    const listRef = useRef<HTMLDivElement>(null);

    // 测量宽度用的ref
    const buttonMeasureRef = useRef<HTMLSpanElement>(null);
    const menuMeasureRef = useRef<HTMLSpanElement>(null);

    const [buttonWidth, setButtonWidth] = useState<number>();
    const [menuContentWidth, setMenuContentWidth] = useState<number>();

    const translatedItems = items.map((item) => t(typeof item === "string" ? item : item?.name));
    const selectedText = t(typeof items[curValue] === "string" ? items[curValue] : items[curValue]?.name);

    // 计算窗口宽度限制
    const finalMaxDropdownWidth =
        typeof window !== "undefined"
            ? Math.min(maxDropdownWidth, window.innerWidth - 32)
            : maxDropdownWidth;

    useLayoutEffect(() => {
        // 只测当前已选项
        if (buttonMeasureRef.current) {
            setButtonWidth(buttonMeasureRef.current.offsetWidth + 20); // padding
        }
        // 菜单宽度测量最长项
        if (menuMeasureRef.current) {
            setMenuContentWidth(menuMeasureRef.current.offsetWidth + 32);
        }
    }, [selectedText, translatedItems.join(","), open, t]);

    useEffect(() => {
        if (!open) return;

        const handleClickOutside = (e: MouseEvent) => {
            const target = e.target as Node;
            if (!btnRef.current?.contains(target) && !listRef.current?.contains(target)) {
                setOpen(false);
            }
        };

        document.addEventListener("click", handleClickOutside);
        return () => document.removeEventListener("click", handleClickOutside);
    }, [open]);

    useEffect(() => {
        setValue(defaultValue);
    }, [defaultValue]);

    // 计算弹窗位置和宽度，防止超出
    const updateDropdownPosition = () => {
        if (btnRef.current) {
            const rect = btnRef.current.getBoundingClientRect();
            // 弹窗宽度
            let width = Math.max(buttonWidth || 0, Math.min(menuContentWidth || 0, finalMaxDropdownWidth));
            // 窗口宽度
            const viewportWidth = window.innerWidth;
            // 如果超出右边界，左移
            let left = rect.left;
            if (left + width > viewportWidth - 8) {
                left = Math.max(8, viewportWidth - width - 8);
            }
            let marginTop = -(currentRef.current?.getBoundingClientRect()?.top ?? 0) +
                (listRef.current?.getBoundingClientRect()?.top ?? 0)
            if (rect.top + marginTop < 0) marginTop = 0
            setDropdownPosition({
                left,
                top: rect.top,
                marginTop
            });
        }
    };

    useEffect(() => {
        updateDropdownPosition();
        window.addEventListener("resize", updateDropdownPosition);
        window.addEventListener("scroll", updateDropdownPosition, true);
        return () => {
            window.removeEventListener("resize", updateDropdownPosition);
            window.removeEventListener("scroll", updateDropdownPosition, true);
        };
    }, [curValue, open, buttonWidth, menuContentWidth, finalMaxDropdownWidth]);

    return (
        <>
            {/* 只测已选项文本宽度（按钮用） */}
            <span
                ref={buttonMeasureRef}
                style={{
                    position: "absolute",
                    visibility: "hidden",
                    whiteSpace: "nowrap",
                    fontSize: "16px",
                    fontFamily: "inherit",
                    fontWeight: "normal",
                    pointerEvents: "none",
                    zIndex: -1,
                    padding: "8px 16px",
                }}
            >
                {selectedText}
            </span>
            {/* 测量所有项中最长一项（菜单用） */}
            <span
                ref={menuMeasureRef}
                style={{
                    position: "absolute",
                    visibility: "hidden",
                    whiteSpace: "nowrap",
                    fontSize: "16px",
                    fontFamily: "inherit",
                    fontWeight: "normal",
                    pointerEvents: "none",
                    zIndex: -1,
                    padding: "8px 16px",
                }}
            >
                {translatedItems.reduce((a, b) => (a.length > b.length ? a : b), "")}
            </span>

            {/* 按钮宽度自适应当前选项 */}
            <AppleButtonWrapper padding={5}>
                <Button
                    appearance="transparent"
                    {...props}
                    icon={<ChevronDownRegular />}
                    iconPosition="after"
                    style={{
                        justifyContent: "space-between",
                        background: "#ffffff11",
                        border: "var(--strokeWidthThin) solid #00000011",
                        width: buttonWidth ? `${buttonWidth}px` : undefined,
                        minWidth: buttonWidth ? `${buttonWidth}px` : undefined,
                    }}
                    ref={btnRef}
                    onClick={(e) => {
                        e.stopPropagation();
                        setOpen((prev) => !prev);
                    }}
                >
                    {selectedText}
                </Button>
            </AppleButtonWrapper>

            {/* 下拉菜单：可比按钮更宽，但不会超出最大宽度和窗口 */}
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
                        position: "fixed",
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
                        minWidth: buttonWidth ? `${buttonWidth}px` : undefined,
                        maxWidth: finalMaxDropdownWidth
                            ? `${finalMaxDropdownWidth}px`
                            : undefined,
                    }}
                    selectedValue={curValue}
                >
                    {items.map((item, index) => {
                        let name = t(typeof item === "string" ? item : item?.name);
                        return <Tab
                            className={styles.tab}
                            ref={(() => {
                                if (index === curValue) return currentRef;
                            })()}
                            key={index}
                            value={index}
                            style={{
                                background:
                                    index === curValue
                                        ? "color-mix(in srgb, var(--colorNeutralBackground2Hover), transparent 10%)"
                                        : "",
                            }}
                            icon={typeof item === "string" ? undefined : item?.icon}
                        >
                            {name}
                        </Tab>
                    }
                    )}
                </TabList>
            </Portal>
        </>
    );
}