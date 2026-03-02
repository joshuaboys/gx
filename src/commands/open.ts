import { ProjectIndex } from "../lib/index.ts";
import { fuzzyMatch, AUTO_JUMP_THRESHOLD } from "../lib/fuzzy.ts";
import type { Config } from "../types.ts";
import { which } from "bun";

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

export function resolveEditor(
  config: Config,
  override?: string,
): { bin: string; args: string[] } {
  const raw =
    override ||
    config.editor ||
    process.env.VISUAL ||
    process.env.EDITOR ||
    "nano";
  const parts = raw.split(/\s+/).filter(Boolean);
  return { bin: parts[0] ?? "nano", args: parts.slice(1) };
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

  const editor = resolveEditor(config, editorOverride);
  const editorInfo = EDITORS[editor.bin] ?? { cmd: editor.bin, gui: false };
  const spawnArgs = [editorInfo.cmd, ...editor.args, projectPath];

  // Verify editor binary exists
  if (!which(editorInfo.cmd)) {
    console.error(`Editor '${editorInfo.cmd}' not found on PATH`);
    process.exit(1);
  }

  if (editorInfo.gui) {
    // Spawn detached for GUI editors — don't block the terminal
    Bun.spawn(spawnArgs, {
      stdout: "ignore",
      stderr: "ignore",
      stdin: "ignore",
    });
    console.error(`Opened ${projectPath} in ${editor.bin}`);
  } else {
    // Terminal editor: inherit stdio for interactive use
    const proc = Bun.spawn(spawnArgs, {
      stdout: "inherit",
      stderr: "inherit",
      stdin: "inherit",
    });
    await proc.exited;
  }
}
