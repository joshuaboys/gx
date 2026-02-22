import { ProjectIndex } from "../lib/index.ts";
import { fuzzyMatch } from "../lib/fuzzy.ts";
import type { Config } from "../types.ts";
import { which } from "bun";

const AUTO_JUMP_THRESHOLD = 0.85;

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
    let resolved = idx.resolve(name);

    // Fuzzy matching fallback (consistent with resolve command)
    if (!resolved) {
      const entries = idx.list().map((e) => ({ name: e.name, path: e.path }));
      const matches = fuzzyMatch(name, entries, config.similarityThreshold);

      const first = matches[0];
      if (matches.length === 1 && first && first.score >= AUTO_JUMP_THRESHOLD) {
        console.error(
          `Fuzzy match: '${name}' -> '${first.name}' (${(first.score * 100).toFixed(0)}%)`,
        );
        resolved = first.path;
      } else if (matches.length > 0) {
        console.error(`No exact match for '${name}'. Did you mean:`);
        for (const [i, m] of matches.entries()) {
          console.error(
            `  ${i + 1}. ${m.name} (${(m.score * 100).toFixed(0)}%)`,
          );
        }
        process.exit(1);
      } else {
        console.error(`Project '${name}' not found`);
        process.exit(1);
      }
    }
    projectPath = resolved;
  } else {
    projectPath = process.cwd();
  }

  const editorName = resolveEditor(config, editorOverride);
  const editorInfo = EDITORS[editorName] ?? { cmd: editorName, gui: false };

  // Verify editor binary exists
  if (!which(editorInfo.cmd)) {
    console.error(`Editor '${editorInfo.cmd}' not found on PATH`);
    process.exit(1);
  }

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
