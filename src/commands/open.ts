import { ProjectIndex } from "../lib/index.ts";
import type { Config } from "../types.ts";

export const EDITORS: Record<string, { cmd: string; gui: boolean }> = {
  code: { cmd: "code", gui: true },
  cursor: { cmd: "cursor", gui: true },
  zed: { cmd: "zed", gui: true },
  vim: { cmd: "vim", gui: false },
  nvim: { cmd: "nvim", gui: false },
  nano: { cmd: "nano", gui: false },
  emacs: { cmd: "emacs", gui: false },
  "emacs-gui": { cmd: "emacs", gui: true },
  subl: { cmd: "subl", gui: true },
};

export function resolveEditor(config: Config, override?: string): string {
  if (override) return override;
  if (config.editor) return config.editor;
  if (process.env.VISUAL) return process.env.VISUAL;
  if (process.env.EDITOR) return process.env.EDITOR;
  return "nano";
}

export async function openProject(
  name: string | undefined,
  config: Config,
  indexPath: string,
  editorOverride?: string,
): Promise<void> {
  let projectPath: string;

  if (name) {
    const idx = await ProjectIndex.load(indexPath);
    const resolved = idx.resolve(name);
    if (!resolved) {
      console.error(`Project '${name}' not found`);
      process.exit(1);
    }
    projectPath = resolved;
  } else {
    projectPath = process.cwd();
  }

  const editorName = resolveEditor(config, editorOverride);
  const editorInfo = EDITORS[editorName] ?? { cmd: editorName, gui: false };

  if (editorInfo.gui) {
    // Spawn detached for GUI editors — don't block the terminal
    Bun.spawn([editorInfo.cmd, projectPath], {
      stdout: "ignore",
      stderr: "ignore",
      stdin: "ignore",
    });
    console.error(`Opened ${projectPath} in ${editorName}`);
  } else {
    // Terminal editor: inherit stdio for interactive use
    const proc = Bun.spawn([editorInfo.cmd, projectPath], {
      stdout: "inherit",
      stderr: "inherit",
      stdin: "inherit",
    });
    await proc.exited;
  }
}
