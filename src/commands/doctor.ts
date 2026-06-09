import { dirname } from "path";
import { access, readFile } from "fs/promises";
import { effectiveProjectDir } from "../lib/config.ts";
import { ProjectIndex } from "../lib/index.ts";
import type { Config } from "../types.ts";

type Status = "ok" | "warn";

interface Check {
  name: string;
  status: Status;
  message: string;
}

async function exists(path: string): Promise<boolean> {
  try {
    await access(path);
    return true;
  } catch {
    return false;
  }
}

function shellRcFile(shellPath: string): string | null {
  const home = process.env.HOME ?? "";
  if (!home) return null;
  const shell = shellPath.split("/").pop() ?? shellPath;
  switch (shell) {
    case "zsh":
      return `${home}/.zshrc`;
    case "bash":
      if (process.platform === "darwin") {
        return `${home}/.bash_profile`;
      }
      return `${home}/.bashrc`;
    case "fish":
      return `${home}/.config/fish/conf.d/gx.fish`;
    default:
      return null;
  }
}

async function shellCheck(): Promise<Check> {
  const shell = process.env.GX_SHELL_OVERRIDE ?? process.env.SHELL ?? "";
  const rc = shellRcFile(shell);
  if (!rc) {
    return {
      name: "shell",
      status: "warn",
      message: "unsupported shell; run gx shell-init <zsh|bash|fish>",
    };
  }

  try {
    const text = await readFile(rc, "utf8");
    if (text.includes("gx shell-init") || text.includes("gx.plugin.zsh")) {
      return {
        name: "shell",
        status: "ok",
        message: `integration found in ${rc}`,
      };
    }
  } catch {
    // Missing shell rc files are common on fresh machines.
  }

  return {
    name: "shell",
    status: "warn",
    message: `integration not found in ${rc}; run gx shell-init`,
  };
}

export async function doctor(
  configPath: string,
  indexPath: string,
  config: Config,
): Promise<void> {
  const projectDir = effectiveProjectDir(config);
  const idx = await ProjectIndex.load(indexPath);
  const entries = idx.list();
  const stale = (
    await Promise.all(
      entries.map(async (entry) => ((await exists(entry.path)) ? null : entry)),
    )
  ).filter((entry) => entry !== null);

  const checks: Check[] = [
    {
      name: "runtime",
      status: "ok",
      message: `bun ${Bun.version}`,
    },
    Bun.which("gx")
      ? {
          name: "binary",
          status: "ok",
          message: `gx found at ${Bun.which("gx")}`,
        }
      : {
          name: "binary",
          status: "warn",
          message: "gx is not on PATH",
        },
    (await exists(configPath))
      ? {
          name: "config",
          status: "ok",
          message: configPath,
        }
      : {
          name: "config",
          status: "warn",
          message: `missing; defaults are in use (${configPath})`,
        },
    (await exists(projectDir))
      ? {
          name: "projectDir",
          status: "ok",
          message: projectDir,
        }
      : {
          name: "projectDir",
          status: "warn",
          message: `missing: ${projectDir}`,
        },
    (await exists(indexPath))
      ? {
          name: "index",
          status: stale.length === 0 ? "ok" : "warn",
          message:
            stale.length === 0
              ? `${entries.length} project(s), ${indexPath}`
              : `${entries.length} project(s), ${stale.length} stale path(s), ${indexPath}`,
        }
      : {
          name: "index",
          status: "warn",
          message: `missing; run gx rebuild (${indexPath})`,
        },
    await shellCheck(),
  ];

  if (!(await exists(dirname(configPath)))) {
    checks.push({
      name: "configDir",
      status: "warn",
      message: `missing: ${dirname(configPath)}`,
    });
  }

  const width = Math.max(...checks.map((check) => check.name.length));
  for (const check of checks) {
    console.log(
      `${check.name.padEnd(width)}  ${check.status.padEnd(4)}  ${check.message}`,
    );
  }

  const warnings = checks.filter((check) => check.status === "warn").length;
  console.log(
    warnings === 0
      ? "doctor: ok"
      : `doctor: ${warnings} warning(s); gx can still run`,
  );
}
