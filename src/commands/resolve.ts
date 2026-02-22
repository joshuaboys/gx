import { ProjectIndex } from "../lib/index.ts";

export async function resolve(
  name: string,
  indexPath: string,
  listAll = false
): Promise<void> {
  const idx = await ProjectIndex.load(indexPath);
  if (listAll) {
    console.log(idx.names().join("\n"));
    return;
  }
  const path = idx.resolve(name);
  if (path) {
    console.log(path);
  } else {
    console.error(`Project '${name}' not found`);
    process.exit(1);
  }
}
