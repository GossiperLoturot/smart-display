import * as z from "zod";

export type Polling = {
  imageUrl: string;
};

export type ConfigEntry = {
  imageUrl: string;
  durationSecs: number;
};

export type Config = {
  entries: ConfigEntry[];
};

export type AppState = {
  config: Config;
  imageIndex: number;
  lastDate: Date;
};

export const PollingScheme = z.object({
  imageUrl: z.string(),
});

export const ConfigEntryScheme = z.object({
  imageUrl: z.string(),
  durationSecs: z.number(),
});

export const ConfigScheme = z.object({
  entries: z.array(ConfigEntryScheme),
});

export const AppStateScheme = z.object({
  config: ConfigScheme,
  imageIndex: z.number(),
  lastDate: z.date(),
});
