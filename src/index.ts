#!/usr/bin/env bun
import { join } from "path";
import { homedir } from "os";
import { loadConfig, getConfigPath, getAgent } from "./lib/config.ts";
import { cloneRepo } from "./commands/clone.ts";
import { ls } from "./commands/ls.ts";
import { resolve } from "./commands/resolve.ts";
import { rebuild } from "./commands/rebuild.ts";
import { showConfig, setConfig } from "./commands/config.ts";
import { openProject } from "./commands/open.ts";
import { initAgent } from "./commands/init.ts";
import { shellInit } from "./commands/shell-init.ts";
import { indexRepos } from "./commands/index-repos.ts";
import { recent } from "./commands/recent.ts";
import { resume } from "./commands/resume.ts";
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
  gx <name> wt [args...]   Jump to project and run wt (worktrunk)
  gx clone <repo>          Clone and jump to repo
  gx open [name]           Open project in editor
  gx ls                    List indexed projects
  gx index                 Index new repos (additive scan)
  gx index <path>...       Add specific repo(s) to index
  gx rebuild               Rescan and rebuild index
  gx recent              List recently visited projects
  gx recent -n <N>       Show last N projects
  gx resume <name>       Jump to project with git context
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
  const agent = getAgent();
  if (agent) console.error(`[gx agent: ${agent}]`);

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
    case "index": {
      const paths = args.slice(1);
      await indexRepos(paths, config, indexPath);
      break;
    }
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
          i > 0 && a !== "--editor" && (editorFlag < 0 || i !== editorFlag + 1),
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
    case "recent": {
      const nFlag = args.indexOf("-n");
      let limit: number | undefined;
      const nValue = nFlag >= 0 ? args[nFlag + 1] : undefined;
      if (nValue) {
        limit = parseInt(nValue, 10);
        if (isNaN(limit) || limit < 1) {
          console.error("Usage: gx recent [-n <count>]");
          process.exit(1);
        }
      }
      await recent(indexPath, limit);
      break;
    }
    case "resume": {
      const name = args[1];
      if (!name) {
        console.error("Usage: gx resume <name>");
        process.exit(1);
      }
      await resume(name, indexPath, config);
      break;
    }
    default:
      // Default: treat as project name to resolve
      await resolve(command, indexPath, config);
      if (process.stdout.isTTY) {
        console.error(
          `Hint: add shell integration for cd support: run 'gx shell-init' and follow the printed instructions for your shell.`,
        );
      }
      break;
  }
}

main().catch((err) => {
  console.error(err.message);
  process.exit(1);
});
