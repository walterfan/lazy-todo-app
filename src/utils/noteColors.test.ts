import { describe, expect, it } from "vitest";
import {
  DEFAULT_NOTE_EDITOR_COLOR,
  getNoteEditorFieldBackground,
  NOTE_COLORS,
} from "./noteColors";

describe("note editor colors", () => {
  it("uses yellow-green as the default editable note background", () => {
    expect(DEFAULT_NOTE_EDITOR_COLOR).toBe("green");
    expect(getNoteEditorFieldBackground(DEFAULT_NOTE_EDITOR_COLOR)).toBe("#9acd32");
  });

  it("keeps the configured note color choices available for the color picker", () => {
    expect(NOTE_COLORS).toEqual(["yellow", "green", "blue", "pink", "purple", "orange"]);
  });
});
