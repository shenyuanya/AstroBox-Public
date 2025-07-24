import { HomeFilled, HomeRegular, PuzzlePieceFilled, PuzzlePieceRegular, SettingsFilled, SettingsRegular, SmartwatchFilled, SmartwatchRegular } from "@fluentui/react-icons";

export const navItems = [
    { name: "/", label: "nav.home", iconRegular: HomeRegular, iconFilled: HomeFilled },
    { name: "/device", label: "nav.device", iconRegular: SmartwatchRegular, iconFilled: SmartwatchFilled },
    { name: "/plugin", label: "nav.plugin", iconRegular: PuzzlePieceRegular, iconFilled: PuzzlePieceFilled },
    { name: "/settings", label: "nav.settings", iconRegular: SettingsRegular, iconFilled: SettingsFilled },
];
export const isNav = (nav: string) => {
    return navItems.some(item => item.name === nav);
}