import { ProjectIndex } from "../lib/index.ts";
import {
  resolveProjectName,
  formatAmbiguous,
  formatAutoMatch,
} from "../lib/resolve-name.ts";
import { CommandError } from "../lib/errors.ts";
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
    const result = resolveProjectName(name, idx, config);

    switch (result.kind) {
      case "exact":
        projectPath = result.path;
        break;

      case "auto":
        console.error(formatAutoMatch(result.query, result.name, result.score));
        projectPath = result.path;
        break;

      case "ambiguous":
        throw new CommandError(formatAmbiguous(result.query, result.matches));

      case "missing":
        throw new CommandError(`Project '${name}' not found`);
    }
  } else {
    projectPath = process.cwd();
  }

  const editorName = resolveEditor(config, editorOverride);
  const editorInfo = EDITORS[editorName] ?? { cmd: editorName, gui: false };

  if (!Bun.which(editorInfo.cmd)) {
    throw new CommandError(`Editor '${editorInfo.cmd}' not found on PATH`);
  }

  if (editorInfo.gui) {
    Bun.spawn([editorInfo.cmd, projectPath], {
      stdout: "ignore",
      stderr: "ignore",
      stdin: "ignore",
    });
    console.error(`Opened ${projectPath} in ${editorName}`);
  } else {
    const proc = Bun.spawn([editorInfo.cmd, projectPath], {
      stdout: "inherit",
      stderr: "inherit",
      stdin: "inherit",
    });
    await proc.exited;
  }
}
