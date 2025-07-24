import { Button, Dropdown, Input, Option } from "@fluentui/react-components";
import styles from "./PluginNodeUI.module.css";
import { PluginUINode } from "@/plugin/types";

export interface PluginNodeUIProps {
  nodes: PluginUINode[];
  onCallback?: (id: string, value?: string) => void;
}

export default function PluginNodeUI({ nodes, onCallback }: PluginNodeUIProps) {
  return (
    <div className={styles.container}>
      {nodes
        .filter((n) => n.visibility)
        .map((node) => {
          const { node_id, disabled, content } = node;
          switch (content.type) {
            case "Text":
              return (
                <p key={node_id} className={styles.text}>
                  {content.value}
                </p>
              );
            case "Button":
              return (
                <Button
                  key={node_id}
                  appearance={content.value.primary ? "primary" : "secondary"}
                  disabled={disabled}
                  className={styles.button}
                  onClick={() => onCallback?.(content.value.callback_fun_id)}
                >
                  {content.value.text}
                </Button>
              );
            case "Dropdown":
              return (
                <Dropdown
                  key={node_id}
                  disabled={disabled}
                  className={styles.dropdown}
                  onOptionSelect={(_, data) =>
                    onCallback?.(content.value.callback_fun_id, data.optionValue?.toString())
                  }
                >
                  {content.value.options.map((opt) => (
                    <Option key={opt} value={opt}>
                      {opt}
                    </Option>
                  ))}
                </Dropdown>
              );
            case "Input":
              return (
                <Input
                  key={node_id}
                  defaultValue={content.value.text}
                  disabled={disabled}
                  className={styles.input}
                  onBlur={(e) => onCallback?.(content.value.callback_fun_id, e.target.value)}
                />
              );
            case "HtmlDocument":
              return (
                <div
                  key={node_id}
                  className={styles.html}
                  dangerouslySetInnerHTML={{ __html: content.value }}
                />
              );
            default:
              return null;
          }
        })}
    </div>
  );
}
