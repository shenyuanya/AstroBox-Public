import { Switch } from "@fluentui/react-components";
import AppleButtonWrapper from "../appleButtonWapper/appleButtonWapper";
import SettingsCard from "./settingsCard";

export default function SwitchCard({ defaultValue=false, onChange, ...props }: { title?: string, desc?: string, defaultValue?: boolean, onChange?: (item: boolean) => void, Icon?: React.ComponentType<any> }) {
    return <SettingsCard {...props}>
        <AppleButtonWrapper>
            <Switch defaultChecked={defaultValue} onChange={(e)=>{onChange?.(e.currentTarget.checked);}} />
        </AppleButtonWrapper>
    </SettingsCard>
}