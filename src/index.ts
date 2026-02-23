#!/usr/bin/env bun
import { join } from "path";
import { homedir } from "os";
import { loadConfig, getConfigPath } from "./lib/config.ts";
import { cloneRepo } from "./commands/clone.ts";
import { ls } from "./commands/ls.ts";
import { resolve } from "./commands/resolve.ts";
import { rebuild } from "./commands/rebuild.ts";
import { showConfig, setConfig } from "./commands/config.ts";
import { openProject } from "./commands/open.ts";
import { initAgent } from "./commands/init.ts";
import { shellInit } from "./commands/shell-init.ts";
import pkg from "../package.json";

const VERSION = pkg.version;

function getIndexPath(): string {
  return join(homedir(), ".config", "gx", "index.json");
}

async function main() {
  const args = process.argv.slice(2);
  const command = args[0];

  if (!command || command === "--help" || command === "-h") {
    console.log(`gx v${VERSION} — git project manager

Usage:
  gx <name>                Jump to project
  gx clone <repo>          Clone and jump to repo
  gx open [name]           Open project in editor
  gx ls                    List indexed projects
  gx rebuild               Rescan and rebuild index
  gx config                Show config
  gx config set <key> <v>  Set config value
  gx init                  Scaffold .claude/ agent config
  gx shell-init [shell]    Print shell integration code
  gx resolve <name>        Resolve project name to path
  gx resolve --list        List all project names

Options:
  gx open --editor <name>  Override editor for this invocation
  gx init --type <type>    Override project type detection
  gx init --force          Overwrite existing CLAUDE.md`);
    return;
  }

  if (command === "--version" || command === "-v") {
    console.log(`gx v${VERSION}`);
    return;
  }

  const configPath = getConfigPath();
  const indexPath = getIndexPath();
  const config = await loadConfig(configPath);

  switch (command) {
    case "clone": {
      const repo = args[1];
      if (!repo) {
        console.error("Usage: gx clone <repo>");
        process.exit(1);
      }
      const path = await cloneRepo(repo, config, indexPath);
      console.log(path);
      break;
    }
    case "ls":
      await ls(indexPath);
      break;
    case "rebuild":
      await rebuild(config, indexPath);
      break;
    case "config":
      if (args[1] === "set" && args[2] && args[3]) {
        await setConfig(configPath, args[2], args[3]);
      } else {
        await showConfig(configPath);
      }
      break;
    case "open": {
      const editorFlag = args.indexOf("--editor");
      let editor: string | undefined;
      if (editorFlag >= 0) {
        editor = args[editorFlag + 1];
        if (!editor || editor.startsWith("--")) {
          console.error("Usage: gx open [name] --editor <name>");
          process.exit(1);
        }
      }
      const name = args.find(
        (a, i) =>
          i > 0 &&
          a !== "--editor" &&
          (editorFlag < 0 || i !== editorFlag + 1),
      );
      await openProject(name, config, indexPath, editor);
      break;
    }
    case "init": {
      const typeFlag = args.indexOf("--type");
      let type: string | undefined;
      if (typeFlag >= 0) {
        type = args[typeFlag + 1];
        if (!type || type.startsWith("--")) {
          console.error("Usage: gx init --type <type>");
          process.exit(1);
        }
      }
      const force = args.includes("--force");
      await initAgent(process.cwd(), { type, force });
      break;
    }
    case "shell-init":
      shellInit(args[1]);
      break;
    case "resolve":
      if (args[1] === "--list") {
        await resolve("", indexPath, config, true);
      } else if (args[1]) {
        await resolve(args[1], indexPath, config);
      } else {
        console.error("Usage: gx resolve <name> | gx resolve --list");
        process.exit(1);
      }
      break;
    default:
      // Default: treat as project name to resolve
      await resolve(command, indexPath, config);
      break;
  }
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
