import type { Component } from "vue";
import { Code, FolderOpen, Terminal } from "@lucide/vue";
import type { EditorKind } from "@/types";

/** 打开方式选项元数据 —— 三处共享(OpenWithMenu 下拉 / OpenWithSettings / ProjectActionsMenu) */
export interface OpenWithOption {
  kind: EditorKind;
  icon: Component;
  labelKey: string;
  descKey: string;
}

export const OPEN_WITH_OPTIONS: readonly OpenWithOption[] = [
  {
    kind: "explorer",
    icon: FolderOpen,
    labelKey: "openWith.explorer",
    descKey: "openWith.openInExplorer",
  },
  { kind: "vscode", icon: Code, labelKey: "openWith.vscode", descKey: "openWith.openInVscode" },
  {
    kind: "terminal",
    icon: Terminal,
    labelKey: "openWith.terminal",
    descKey: "openWith.openInTerminal",
  },
] as const;