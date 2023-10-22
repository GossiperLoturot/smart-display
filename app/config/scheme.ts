import * as z from "zod";

export class ConfigEntry {
  imageUrl: string;
  durationSecs: number;
  constructor();
  constructor(imageUrl: string, durationSecs: number);
  constructor(imageUrl?: string, durationSecs?: number) {
    this.imageUrl = imageUrl || "";
    this.durationSecs = durationSecs || 0;
  }
}

export class Config {
  entries: ConfigEntry[];
  constructor();
  constructor(entries: ConfigEntry[]);
  constructor(entries?: ConfigEntry[]) {
    this.entries = entries || [];
  }
}

export const ConfigEntryScheme = z.object({
  imageUrl: z.string(),
  durationSecs: z.number(),
});

export const ConfigScheme = z.object({
  entries: z.array(ConfigEntryScheme),
});
