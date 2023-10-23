import { readFile, writeFile } from "fs/promises";
import { AppState, Config, ConfigScheme, Polling } from "./scheme";
import { add } from "date-fns";

let appState: AppState | undefined = undefined;

export async function useAppState(): Promise<AppState> {
  if (!appState) {
    const config = await readConfig();
    const imageIndex = 0;
    const lastDate = new Date();
    appState = { config, imageIndex, lastDate };
  }
  return appState;
}

export function poll(appState: AppState): Polling {
  const seconds = appState.config.entries[appState.imageIndex].durationSecs;
  const nextDate = add(appState.lastDate, { seconds });
  if (nextDate <= new Date()) {
    appState.imageIndex =
      (appState.imageIndex + 1) % appState.config.entries.length;
    appState.lastDate = nextDate;
  }
  return { imageUrl: appState.config.entries[appState.imageIndex].imageUrl };
}

export async function readConfig(): Promise<Config> {
  const buf = await readFile("smart-display.dyn.json");
  const json = JSON.parse(buf.toString());
  return ConfigScheme.parse(json) as Config;
}

export async function writeConfig(config: Config): Promise<void> {
  const buf = Buffer.from(JSON.stringify(config));
  await writeFile("smart-display.dyn.json", buf);
}
