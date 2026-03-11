import { ProjectIndex } from "../lib/index.ts";
import {
  resolveProjectName,
  formatAmbiguous,
  formatAutoMatch,
} from "../lib/resolve-name.ts";
import { CommandError } from "../lib/errors.ts";
import type { Config } from "../types.ts";

export async function resolve(
  name: string,
  indexPath: string,
  config: Config,
  listAll = false,
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);

  if (listAll) {
    console.log(idx.names().join("\n"));
    return;
  }

  const result = resolveProjectName(name, idx, config);

  switch (result.kind) {
    case "exact":
      idx.touch(result.name);
      await idx.save(indexPath);
      console.log(result.path);
      return;

    case "auto":
      console.error(formatAutoMatch(result.query, result.name, result.score));
      idx.touch(result.name);
      await idx.save(indexPath);
      console.log(result.path);
      return;

    case "ambiguous":
      throw new CommandError(formatAmbiguous(result.query, result.matches));

    case "missing":
      throw new CommandError(`Project '${name}' not found`);
  }
}
