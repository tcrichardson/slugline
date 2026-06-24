/** Mirrors the JSON returned by `GET /api/config` (serde-serialized UiConfig). */
export interface UiConfig {
  theme: string;
  font: string;
  edit_line_position: number;
  colors: Record<string, Record<string, string>>;
}
