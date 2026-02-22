import type { ParsedRepo } from "../types.ts";

const HTTPS_RE = /^https?:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SSH_RE = /^(?:ssh:\/\/)?[^@]+@([^/:]+)(?::\d+)?[:/](.+?)(?:\.git)?\/?$/;
const GIT_RE = /^git:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SHORTHAND_RE = /^([a-zA-Z0-9_.-]+)\/([a-zA-Z0-9_.-][a-zA-Z0-9_.\-/]*)$/;

function validateSegments(owner: string, repo: string): void {
  for (const seg of [...owner.split("/"), repo]) {
    if (seg === ".." || seg === "." || !seg) {
      throw new Error("Path traversal detected");
    }
    if (/[\\\0]/.test(seg)) {
      throw new Error(`Invalid characters in repository path`);
    }
  }
}

export function parseUrl(input: string, defaultHost = "github.com"): ParsedRepo {
  const trimmed = input.trim();
  if (!trimmed) throw new Error("Empty repository URL");

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
    const cleanRepo = repo!.replace(/\/$/, "");
    validateSegments(owner!, cleanRepo);
    return {
      host: defaultHost,
      owner: owner!,
      repo: cleanRepo,
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
  validateSegments(owner, repo);
  return { host, owner, repo, originalUrl };
}

export function toCloneUrl(parsed: ParsedRepo): string {
  if (parsed.originalUrl.startsWith("git@") || parsed.originalUrl.startsWith("ssh://")) {
    return parsed.originalUrl;
  }
  return `https://${parsed.host}/${parsed.owner}/${parsed.repo}.git`;
}
