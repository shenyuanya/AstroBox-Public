import { Button, Toast, Toaster, ToastTitle, ToastTrigger, useToastController } from "@fluentui/react-components"
import { Dismiss12Regular } from "@fluentui/react-icons"
import { ReactNode } from "react"

let id = "toaster"

export function ToastSurface() {
    return <Toaster
        toasterId={id}
        position="top"
        pauseOnHover
        pauseOnWindowBlur
    />
}

export default function useToast(){
    return useToastController(id)
}

export function makeSuceess(dispatchToast: (content: ReactNode, options?: any) => void,message: string){
    return dispatchToast(
        <Toast>
            <ToastTitle action={
                <ToastTrigger>
                    <Button appearance="transparent" size="small" icon={<Dismiss12Regular />}></Button>
                </ToastTrigger>
            }>{message}</ToastTitle>
        </Toast>,{intent:"success"}
    )
}
export function makeError(dispatchToast: (content: ReactNode, options?: any) => void,message: string){
    return dispatchToast(
        <Toast>
            <ToastTitle action={
                <ToastTrigger>
                    <Button appearance="transparent" size="small" icon={<Dismiss12Regular />}></Button>
                </ToastTrigger>
            }>{message}</ToastTitle>
        </Toast>,{intent:"error"}
    )
}