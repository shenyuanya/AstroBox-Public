import { useI18n } from "@/i18n";
import {
  Body1,
  Button,
  Dialog,
  DialogActions,
  DialogBody,
  DialogContent,
  DialogSurface,
  DialogTitle,
  DialogTrigger,
} from "@fluentui/react-components";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";

export function DisclaimerDialog({
  defaultOpen = false,
  trigger,
}: {
  defaultOpen?: boolean;
  trigger?: React.ReactElement;
}) {
  const { t } = useI18n();
  const [canClose, setCanClose] = useState(false);
  const contentRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const el = contentRef.current;
    if (el && el.scrollHeight <= el.clientHeight) {
      setCanClose(true);
    }
  }, []);

  const handleScroll = (e: React.UIEvent<HTMLElement>) => {
    const el = e.currentTarget;
    const reachedBottom =
      el.scrollTop + el.clientHeight >= el.scrollHeight - 2;
    if (reachedBottom) setCanClose(true);
  };

  return (
    <Dialog defaultOpen={defaultOpen} modalType="alert">
      <DialogTrigger>{trigger}</DialogTrigger>

      <DialogSurface>
        <DialogBody>
          <DialogTitle>{t("disclaimer.title")}</DialogTitle>

          <DialogContent
            ref={contentRef}
            onScroll={handleScroll}
            style={{ maxHeight: "60vh", overflowY: "auto" }}
          >
            <Body1>
              {t("disclaimer.content")
                .split("\n")
                .map((line, idx) => (
                  <p key={idx}>{line}</p>
                ))}
            </Body1>
          </DialogContent>

          <DialogActions>
            <DialogTrigger action="close">
              <Button
                appearance="primary"
                disabled={!canClose}
                onClick={() => localStorage.setItem("disclaimerAccepted", "true")}
              >
                {t("disclaimer.confirm")}
              </Button>
            </DialogTrigger>

            <Button onClick={() => invoke("cleanup_before_exit")}>
              {t("disclaimer.cancel")}
            </Button>
          </DialogActions>
        </DialogBody>
      </DialogSurface>
    </Dialog>
  );
}