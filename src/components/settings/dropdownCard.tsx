import SettingsCard from "./settingsCard";
import WinDropdown from "./winDropdown";

export default function DropdownCard({ title, desc, items, onSelect, Icon, defaultValue }: { title?: string, desc?: string, defaultValue: number, items: string[], onSelect?: (item: number) => void, Icon?: React.ComponentType<any> }) {

    return <SettingsCard title={title} desc={desc} Icon={Icon}>
        <WinDropdown
            items={items}
            onValueChange={onSelect}
            /*@ts-ignore */
            style={{ background: "color-mix(in oklch, var(--colorNeutralBackground1), transparent 10%);"}}
            defaultValue={defaultValue}
        />
    </SettingsCard>
}