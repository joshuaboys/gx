import { join } from "path";
import { mkdir } from "fs/promises";
import { detectProjectType, VALID_TYPES, type ProjectType } from "../lib/detect.ts";
import { getClaudeMd, getPlanCommand, getReviewCommand } from "../lib/templates.ts";

export async function initAgent(
  dir: string,
  opts: { type?: string; force?: boolean },
): Promise<void> {
  // 1. Determine project type
  let projectType: ProjectType;
  if (opts.type) {
    if (!VALID_TYPES.has(opts.type)) {
      throw new Error(
        `Unknown project type '${opts.type}'. Valid types: ${[...VALID_TYPES].join(", ")}`,
      );
    }
    projectType = opts.type as ProjectType;
  } else {
    projectType = await detectProjectType(dir);
  }

  // 2. Check if .claude/CLAUDE.md already exists
  const claudeDir = join(dir, ".claude");
  const claudeMdPath = join(claudeDir, "CLAUDE.md");
  const commandsDir = join(claudeDir, "commands");

  if (!opts.force && (await Bun.file(claudeMdPath).exists())) {
    throw new Error(
      `.claude/CLAUDE.md already exists. Use --force to overwrite.`,
    );
  }

  // 3. Create directories
  await mkdir(commandsDir, { recursive: true });

  // 4. Write files
  await Bun.write(claudeMdPath, getClaudeMd(projectType));
  await Bun.write(join(commandsDir, "plan.md"), getPlanCommand());
  await Bun.write(join(commandsDir, "review.md"), getReviewCommand());

  // 5. Report what was created
  console.error(`Scaffolded .claude/ for ${projectType} project:`);
  console.error(`  .claude/CLAUDE.md`);
  console.error(`  .claude/commands/plan.md`);
  console.error(`  .claude/commands/review.md`);
}
