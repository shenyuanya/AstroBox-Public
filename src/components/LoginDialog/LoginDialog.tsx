import useToast, { makeSuceess } from "@/layout/toast";
import {
    Avatar,
    Button,
    Checkbox,
    Dialog,
    DialogBody,
    DialogContent,
    DialogSurface,
    DialogTitle,
    Field,
    Input,
    Spinner
} from "@fluentui/react-components";
import { ArrowRight16Filled, Key20Regular, Person24Filled, PersonRegular } from "@fluentui/react-icons";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { openUrl } from "@tauri-apps/plugin-opener";
import { useId, useRef, useState } from "react";
import { useI18n } from "@/i18n";

interface ProviderDialogProps {
    open: boolean;
    onClose: () => void;
}
export default function LoginDialog({ open, onClose }: ProviderDialogProps) {
    const [succ, setSucc] = useState<boolean>(true);
    const [msg, setMsg] = useState<string>("");
    const { dispatchToast } = useToast();
    const { t } = useI18n();

    const [user, setUser] = useState<string>(localStorage.getItem("mi_acc_usr") ?? "");
    const [pass, setPass] = useState<string>(localStorage.getItem("mi_acc_pwd") ?? "");
    const [running, setRunning] = useState<boolean>(false);
    const saveRef = useRef<HTMLInputElement>(null);

    const [need2fa, setNeed2fa] = useState(false);
    const [pending2faUrl, setPending2faUrl] = useState("");
    const [ua, setUA] = useState(navigator.userAgent);

    const getUa = async (): Promise<string> => {
        openUrl("https://astrobox.online/uacallback");

        return new Promise<string>((resolve, reject) => {
            let finished = false;
            const timeout = setTimeout(() => {
                finished = true;
                reject(new Error("获取UA超时"));
            }, 15000);

            const unlistenPromise = listen("ua-callback", (data) => {
                if (finished) return;
                finished = true;
                clearTimeout(timeout);
                try {
                    const payload = data.payload as any;
                    const ua = payload.ua;
                    resolve(ua);
                } finally {
                    unlistenPromise.then(unlisten => unlisten());
                }
            });
        });
    };

    const Login2faTasks = async () => {
        setNeed2fa(false);

        let ua = await getUa();
        setUA(ua);

        // 哥们给点反应时间以防止出现奇怪的问题
        setTimeout(() => {
            openUrl(pending2faUrl);
        }, 200)
    }

    const Login = async () => {
        setRunning(true)
        if (user == "" || pass == "") {
            setSucc(false)
            setRunning(false)
            setMsg(t('loginDialog.emptyError'))
            return
        }
        const savePwd = saveRef?.current?.checked ?? false
        localStorage.setItem("mi_acc_save", savePwd.toString())
        if (savePwd) {
            localStorage.setItem("mi_acc_pwd", pass)
            localStorage.setItem("mi_acc_usr", user)
        } else {
            localStorage.removeItem("mi_acc_save")
            localStorage.removeItem("mi_acc_pwd")
            localStorage.removeItem("mi_acc_usr")
        }

        let result = await invoke<any>("get_login_mi_account", { "username": user, "password": pass, "ua": ua }).catch(e => { return e });

        console.log(result)

        if (typeof result == "string") {
            setSucc(false)
            setRunning(false)

            if (result.includes("2-f-a=")) {
                let url = result.split("2-f-a=")[1];
                setPending2faUrl(url);
                setNeed2fa(true);
                setSucc(false);
                setMsg(t('loginDialog.2fa.body'));
                return;
            }

            setMsg(result)
            console.log("get_login_mi_account fail!")
            console.log(result)
            return
        } else {
            let device = await invoke<any>("get_mi_device_list", { "token": result, "ua": ua }).catch(e => { return e });

            if (typeof device == "string") {
                setSucc(false)
                setRunning(false)
                setMsg(device)

                console.log("get_mi_device_list fail!")

                return
            } else {
                console.log(device)
                setSucc(true)

                let array = device["result"]["list"]
                array.forEach(async (element: any) => {
                    if(element.detail.encrypt_key){
                        await invoke<any>("add_new_device", { "name": element["name"], "addr": element["detail"]["mac"], "authkey": element["detail"]["encrypt_key"] }).catch(() => null);
                    }
                });

                makeSuceess(dispatchToast, "绑定设备同步成功!")
                setRunning(false)
                onClose()
            }
        }

    };
    const userNameId = useId();
    const passWordId = useId();

    return (
        <>
            <Dialog open={open} modalType="alert" onOpenChange={(_, d) => !d.open && onClose()}>
                <DialogSurface style={{ maxWidth: "468px" }}>
                    <DialogBody style={{ display: "flex", flexDirection: "column", alignItems: "center" }}>
                        <DialogTitle style={{
                            display: "flex",
                            flexDirection: "column",
                            alignItems: "center",
                            gap: "4px"
                        }}>
                            <Avatar size={48} color="peach" icon={<Person24Filled />} />
                            {t('loginDialog.title')}
                        </DialogTitle>
                        <DialogContent style={{ width: "100%", padding: "0" }}>
                        <Field label={t('loginDialog.description')} style={{ width: "100%", gap: '8px', display: 'flex', flexDirection: 'column', alignItems: 'center', userSelect: 'text' }}
                                validationMessage={msg}
                                validationState={succ ? "none" : "error"}
                            >
                                <Input
                                    onChange={(ev, data) => {
                                        setUser(data.value)
                                    }}
                                    value={user}
                                    id={userNameId}
                                    placeholder={t('loginDialog.usernamePlaceholder')}
                                    size="large"
                                    contentBefore={<PersonRegular fontSize={20} style={{ width: "20px" }} />}
                                    style={{ width: "100%", padding: "0 10px", gap: "2px", marginTop: "18px" }}
                                    autoComplete="off"
                                />
                                <Input
                                    type="password"
                                    onChange={(ev, data) => {
                                        setPass(data.value)
                                    }}
                                    id={passWordId}
                                    value={pass}
                                    placeholder={t('loginDialog.passwordPlaceholder')}
                                    size="large"
                                    contentBefore={<Key20Regular fontSize="20px" style={{ width: "20px" }} />}
                                    style={{ width: "100%", padding: "0 10px", gap: "2px" }}
                                    input={{
                                        style: {
                                            paddingRight: "0",
                                        }
                                    }}
                                />
                                <Checkbox
                                    defaultChecked={localStorage.getItem("mi_acc_save") === "true"}
                                    ref={saveRef}
                                    label={t('loginDialog.saveCredentials')}
                                />
                            </Field>
                        </DialogContent>
                        <Button onClick={Login} appearance="primary" size="large" style={{ width: '100%', display: 'flex', fontWeight: '500', gap: '8px', marginTop: '20px', fontSize: '15px', ...(running ? { pointerEvents: 'none', opacity: 0.75 } : {}) }}>{running ? <><Spinner size="extra-tiny" appearance="inverted" /> {t('loginDialog.loggingIn')}</> : <>{t('loginDialog.login')} <ArrowRight16Filled /></>}</Button>
                        <Button onClick={onClose} appearance="transparent">{t('common.cancel')}</Button>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
            <Dialog open={need2fa} onOpenChange={(_, d) => !d.open && setNeed2fa(false)}>
                <DialogSurface>
                    <DialogBody>
                        <DialogTitle>{t('loginDialog.2fa.title')}</DialogTitle>
                        <DialogContent>
                            <div style={{ margin: '12px 0' }}>
                                {t('loginDialog.2fa.body')}
                            </div>
                        </DialogContent>
                        <Button
                            appearance="primary"
                            onClick={Login2faTasks}
                            style={{ marginTop: 12, width: '100%' }}
                        >
                            {t('common.ok')}
                        </Button>
                    </DialogBody>
                </DialogSurface>
            </Dialog>
        </>
    );
}