import type { Component } from "vue";
import {
  Anchor,
  Box,
  Bug,
  Cloud,
  Container,
  Cpu,
  Database,
  Flame,
  FlaskConical,
  GitBranch,
  Globe,
  Hammer,
  Layers,
  Package,
  Rocket,
  Server,
  Ship,
  Terminal,
  Toolbox,
  Wrench,
  Zap,
} from "@lucide/vue";

/** 自定义命令可选的开发类图标(lucide) */
export const COMMAND_ICONS: { name: string; component: Component }[] = [
  { name: "rocket", component: Rocket },
  { name: "server", component: Server },
  { name: "database", component: Database },
  { name: "container", component: Container },
  { name: "terminal", component: Terminal },
  { name: "bug", component: Bug },
  { name: "flask-conical", component: FlaskConical },
  { name: "package", component: Package },
  { name: "git-branch", component: GitBranch },
  { name: "cloud", component: Cloud },
  { name: "zap", component: Zap },
  { name: "globe", component: Globe },
  { name: "hammer", component: Hammer },
  { name: "wrench", component: Wrench },
  { name: "flame", component: Flame },
  { name: "layers", component: Layers },
  { name: "cpu", component: Cpu },
  { name: "toolbox", component: Toolbox },
  { name: "ship", component: Ship },
  { name: "anchor", component: Anchor },
  { name: "box", component: Box },
];

const ICON_MAP = new Map(COMMAND_ICONS.map((i) => [i.name, i.component]));

/** 按名字取图标组件,未知名字返回 undefined */
export function commandIcon(name: string): Component | undefined {
  return ICON_MAP.get(name);
}
