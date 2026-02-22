import { join } from "path";
import type { ParsedRepo, Config } from "../types.ts";
import { expandTilde } from "./config.ts";

export function toPath(parsed: ParsedRepo, config: Config): string {
  const base = expandTilde(config.projectDir);
  if (config.structure === "host") {
    return join(base, parsed.host, parsed.owner, parsed.repo);
  }
  return join(base, parsed.owner, parsed.repo);
}
