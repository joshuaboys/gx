import type { ParsedRepo } from "../types.ts";

const HTTPS_RE = /^https?:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SSH_RE = /^(?:ssh:\/\/)?[^@]+@([^/:]+)(?::\d+)?[:/](.+?)(?:\.git)?\/?$/;
const GIT_RE = /^git:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SHORTHAND_RE = /^([a-zA-Z0-9_.-]+)\/([a-zA-Z0-9_.-][a-zA-Z0-9_.\-/]*)$/;

export function parseUrl(input: string, defaultHost = "github.com"): ParsedRepo {
  const trimmed = input.trim();
  if (!trimmed) throw new Error("Empty repository URL");
  if (trimmed.includes("..")) throw new Error("Path traversal detected");

  let host: string;
  let path: string;

  // Try each pattern
  let match = trimmed.match(HTTPS_RE);
  if (match) {
    [, host, path] = match;
    return buildParsed(host!, path!, trimmed);
  }

  match = trimmed.match(GIT_RE);
  if (match) {
    [, host, path] = match;
    return buildParsed(host!, path!, trimmed);
  }

  match = trimmed.match(SSH_RE);
  if (match) {
    [, host, path] = match;
    return buildParsed(host!, path!, trimmed);
  }

  match = trimmed.match(SHORTHAND_RE);
  if (match) {
    const [, owner, repo] = match;
    return {
      host: defaultHost,
      owner: owner!,
      repo: repo!.replace(/\/$/, ""),
      originalUrl: `https://${defaultHost}/${owner}/${repo}`,
    };
  }

  throw new Error(`Cannot parse repository URL: ${trimmed}`);
}

function buildParsed(host: string, path: string, originalUrl: string): ParsedRepo {
  const segments = path.split("/");
  if (segments.length < 2) {
    throw new Error(`Invalid repository path: ${path}`);
  }
  const repo = segments.pop()!;
  const owner = segments.join("/");
  return { host, owner, repo, originalUrl };
}

export function toCloneUrl(parsed: ParsedRepo): string {
  return parsed.originalUrl.startsWith("git@")
    ? parsed.originalUrl
    : `https://${parsed.host}/${parsed.owner}/${parsed.repo}.git`;
}
