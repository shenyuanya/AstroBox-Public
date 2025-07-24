import BasePage from "@/layout/basePage";
import crate from "@/licences/crate";
import npm from "@/licences/npm";
import { useI18n } from "@/i18n";
import { Input, Tab, TabList } from "@fluentui/react-components";
import { ArrowCircleUp20Filled, Filter20Filled } from '@fluentui/react-icons';
import { openUrl } from "@tauri-apps/plugin-opener";
import Link from "next/link";
import { useMemo, useState } from "react";

export default function Licences() {
    const { t } = useI18n();
    const npmLicenses = useMemo(() => {
        const temp: License[] = []
        for (const license of Object.keys(npm)) {
            //@ts-ignore
            for (const pkg of npm[license]) {
                //@ts-ignore
                temp.push({
                    name: pkg.name,
                    version: pkg.versions?.join(" "),
                    author: pkg.author,
                    license: pkg.license,
                    description: pkg.description,
                    url: pkg.homepage ?? "",
                })
            }
        }
        return temp
    }, [])
    const crateLicense = useMemo(() => crate.filter(license => license.authors !== "AstralSightStudios").map(license => {
        return {
            name: license.name,
            version: license.version,
            author: license.authors ?? "",
            license: license.license ?? "",
            description: license.description,
            url: license.repository ?? "",
        } satisfies License
    }), [])
    const [filter, setFilter] = useState("" as string);
    return (
        <BasePage title={t('licences.title')}>
            <p id="top">{t('licences.intro')}</p>
            <div style={{
                position: "sticky", top: "0px", display: "flex", flexDirection: "row", alignItems: "center", justifyContent: "spaceBetween", flexWrap: "wrap", gap: "10px", background: "var(--cardbackground)",
                borderRadius: "6px", padding: "4px",backdropFilter:"blur(15px)"
            }}>
                <TabList defaultSelectedValue="tab2" appearance="subtle" size="small">
                    <Tab value="tab1"><Link href="#node">{t('licences.node')}</Link></Tab>
                    <Tab value="tab2"><Link href="#crates">{t('licences.crates')}</Link></Tab>
                </TabList>
                <div style={{
                    display: "flex", flexDirection: "row", alignItems: "center", justifyContent: "spaceBetween", gap: "10px"
                }}>
                    <Input style={{ width: '100%' }} contentBefore={<Filter20Filled />} placeholder={t('licences.filter')} value={filter} onChange={(ev, data) => {
                        setFilter(data.value)
                    }} />
                    <Link href="#top" style={{ display: "flex", alignItems: "center", justifyContent: "center", padding: "6px" }}><ArrowCircleUp20Filled /></Link>
                </div>
            </div>
            <h2 id="node">{t('licences.node')}</h2>
            <ul style={{
                paddingInlineStart: "18px",
                marginBlockStart: "0"
            }}>
                {npmLicenses
                    .filter((license) => filterFn(license, filter))
                    .map((license) => (
                        <LicenseCard license={license} key={license.name} />
                    ))}
            </ul>
            <h2 id="crates">{t('licences.crates')}</h2>
            <ul style={{
                paddingInlineStart: "18px",
                marginBlockStart: "0"
            }}>
                {crateLicense
                    .filter((license) => filterFn(license, filter))
                    .map((license) => (
                        <LicenseCard license={license} key={license.name} />
                    ))}
            </ul>
        </BasePage>
    )
}
function filterFn(license: License, filter: string) {
    return license.name?.toLowerCase().includes(filter.toLowerCase()) || license.author?.toLowerCase().includes(filter.toLowerCase()) || license.license?.toLowerCase().includes(filter.toLowerCase()) || license.description?.toLowerCase().includes(filter.toLowerCase()) || license.url?.toLowerCase().includes(filter.toLowerCase())
}
function LicenseCard({ license }: { license: License }) {
    const { t } = useI18n();
    return <li key={license.name}>
        <h3>{license.name} ({license.version})</h3>
        {license.author && <p>{t('licences.author')} {license.author}</p>}
        <p>{t('licences.license')} {license.license}</p>
        {license.description && <p>{t('licences.description')} {license.description}</p>}
        {license.url && <p>{t('licences.url')} <Link href="" onClick={() => {
            openUrl(license.url!)
        }}>{license.url}</Link></p>}
    </li>
}
interface License {
    name: string | undefined;
    version: string | undefined;
    author: string | undefined;
    license: string | undefined;
    description: string | undefined;
    url: string | undefined;
}