import type { ParsedRepo } from "../types.ts";

const HTTPS_RE = /^https?:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SSH_RE = /^(?:ssh:\/\/)?[^@]+@([^/:]+)(?::\d+)?[:/](.+?)(?:\.git)?\/?$/;
const GIT_RE = /^git:\/\/([^/]+)\/(.+?)(?:\.git)?\/?$/;
const SHORTHAND_RE = /^([a-zA-Z0-9_.-]+)\/([a-zA-Z0-9_.-][a-zA-Z0-9_.\-/]*)$/;
const BARE_NAME_RE = /^[a-zA-Z0-9_.-]+$/;

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

function extract(match: RegExpMatchArray): [string, string] {
  return [match[1] ?? "", match[2] ?? ""];
}

export function parseUrl(
  input: string,
  defaultHost = "github.com",
  defaultOwner = "",
): ParsedRepo {
  const trimmed = input.trim();
  if (!trimmed) throw new Error("Empty repository URL");

  let match = trimmed.match(HTTPS_RE);
  if (match) {
    const [host, path] = extract(match);
    return buildParsed(host, path, trimmed);
  }

  match = trimmed.match(GIT_RE);
  if (match) {
    const [host, path] = extract(match);
    return buildParsed(host, path, trimmed);
  }

  match = trimmed.match(SSH_RE);
  if (match) {
    const [host, path] = extract(match);
    return buildParsed(host, path, trimmed);
  }

  match = trimmed.match(SHORTHAND_RE);
  if (match) {
    const [owner, rawRepo] = extract(match);
    const repo = rawRepo.replace(/\/$/, "");
    validateSegments(owner, repo);
    return {
      host: defaultHost,
      owner,
      repo,
      originalUrl: `https://${defaultHost}/${owner}/${rawRepo}`,
    };
  }

  if (BARE_NAME_RE.test(trimmed) && defaultOwner) {
    validateSegments(defaultOwner, trimmed);
    return {
      host: defaultHost,
      owner: defaultOwner,
      repo: trimmed,
      originalUrl: `https://${defaultHost}/${defaultOwner}/${trimmed}`,
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
