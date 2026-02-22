import { test, expect, describe, beforeEach, afterEach } from "bun:test";
import { resolveEditor, EDITORS } from "../../src/commands/open.ts";
import type { Config } from "../../src/types.ts";
import { DEFAULT_CONFIG } from "../../src/types.ts";

describe("resolveEditor", () => {
  const originalVisual = process.env.VISUAL;
  const originalEditor = process.env.EDITOR;

  afterEach(() => {
    // Restore original env vars
    if (originalVisual !== undefined) {
      process.env.VISUAL = originalVisual;
    } else {
      delete process.env.VISUAL;
    }
    if (originalEditor !== undefined) {
      process.env.EDITOR = originalEditor;
    } else {
      delete process.env.EDITOR;
    }
  });

  test("returns override when provided", () => {
    const config: Config = { ...DEFAULT_CONFIG, editor: "vim" };
    expect(resolveEditor(config, "code")).toBe("code");
  });

  test("falls back to config.editor when no override", () => {
    const config: Config = { ...DEFAULT_CONFIG, editor: "nvim" };
    delete process.env.VISUAL;
    delete process.env.EDITOR;
    expect(resolveEditor(config)).toBe("nvim");
  });

  test("falls back to $VISUAL when config.editor is empty", () => {
    const config: Config = { ...DEFAULT_CONFIG, editor: "" };
    process.env.VISUAL = "code";
    delete process.env.EDITOR;
    expect(resolveEditor(config)).toBe("code");
  });

  test("falls back to $EDITOR when $VISUAL is unset", () => {
    const config: Config = { ...DEFAULT_CONFIG, editor: "" };
    delete process.env.VISUAL;
    process.env.EDITOR = "vim";
    expect(resolveEditor(config)).toBe("vim");
  });

  test("falls back to nano when nothing else is set", () => {
    const config: Config = { ...DEFAULT_CONFIG, editor: "" };
    delete process.env.VISUAL;
    delete process.env.EDITOR;
    expect(resolveEditor(config)).toBe("nano");
  });

  test("override takes priority over everything", () => {
    const config: Config = { ...DEFAULT_CONFIG, editor: "emacs" };
    process.env.VISUAL = "code";
    process.env.EDITOR = "vim";
    expect(resolveEditor(config, "zed")).toBe("zed");
  });
});

describe("EDITORS table", () => {
  test("code is a GUI editor", () => {
    expect(EDITORS["code"]).toEqual({ cmd: "code", gui: true });
  });

  test("cursor is a GUI editor", () => {
    expect(EDITORS["cursor"]).toEqual({ cmd: "cursor", gui: true });
  });

  test("zed is a GUI editor", () => {
    expect(EDITORS["zed"]).toEqual({ cmd: "zed", gui: true });
  });

  test("subl is a GUI editor", () => {
    expect(EDITORS["subl"]).toEqual({ cmd: "subl", gui: true });
  });

  test("emacs-gui is a GUI editor using emacs command", () => {
    expect(EDITORS["emacs-gui"]).toEqual({ cmd: "emacs", gui: true });
  });

  test("vim is a terminal editor", () => {
    expect(EDITORS["vim"]).toEqual({ cmd: "vim", gui: false });
  });

  test("nvim is a terminal editor", () => {
    expect(EDITORS["nvim"]).toEqual({ cmd: "nvim", gui: false });
  });

  test("nano is a terminal editor", () => {
    expect(EDITORS["nano"]).toEqual({ cmd: "nano", gui: false });
  });

  test("emacs is a terminal editor", () => {
    expect(EDITORS["emacs"]).toEqual({ cmd: "emacs", gui: false });
  });

  test("unknown editor defaults to terminal mode", () => {
    const editorName = "my-custom-editor";
    const info = EDITORS[editorName] ?? { cmd: editorName, gui: false };
    expect(info).toEqual({ cmd: "my-custom-editor", gui: false });
  });
});
