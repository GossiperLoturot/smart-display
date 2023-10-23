"use client";

import { Dispatch, ReactNode, useEffect, useReducer } from "react";
import { Config, ConfigEntry, ConfigScheme } from "./scheme";

type ConfigAction =
  | { type: "addConfigEntry"; index: number }
  | { type: "removeConfigEntry"; index: number }
  | { type: "modifyConfigEntry"; index: number; configEntry: ConfigEntry }
  | { type: "loadConfig"; config: Config };

function configReducer(
  config: Config | undefined,
  dispatch: ConfigAction,
): Config | undefined {
  switch (dispatch.type) {
    case "addConfigEntry": {
      if (!config) return undefined;
      const newConfig = { ...config, entries: [...config.entries] };
      if (0 <= dispatch.index && dispatch.index <= newConfig.entries.length) {
        newConfig.entries.splice(dispatch.index, 0, new ConfigEntry());
      }
      return newConfig;
    }
    case "removeConfigEntry": {
      if (!config) return undefined;
      const newConfig = { ...config, entries: [...config.entries] };
      if (0 <= dispatch.index && dispatch.index < newConfig.entries.length) {
        newConfig.entries.splice(dispatch.index, 1);
      }
      return newConfig;
    }
    case "modifyConfigEntry": {
      if (!config) return undefined;
      const newConfig = { ...config, entries: [...config.entries] };
      if (0 <= dispatch.index && dispatch.index < newConfig.entries.length) {
        newConfig.entries[dispatch.index] = dispatch.configEntry;
      }
      return newConfig;
    }
    case "loadConfig":
      return dispatch.config;
  }
}

function ConfigForm(): ReactNode {
  const [config, dispatch] = useReducer(configReducer, undefined);

  useEffect(() => {
    fetchConfig()
      .then((config) => {
        console.info("successful to fetch config");
        dispatch({ type: "loadConfig", config });
      })
      .catch((reason) => {
        throw reason;
      });
  }, []);

  if (!config) {
    return (
      <div>
        <p>Loading</p>
      </div>
    );
  }

  const onSubmit = () => {
    submitConfig(config)
      .then(() => {
        console.info("successful to submit config");
      })
      .catch((reason) => {
        throw reason;
      });
  };

  return (
    <form method="dialog" onSubmit={onSubmit}>
      {config.entries.map((_, index) => (
        <ConfigEntryForm
          configEntry={config.entries[index]}
          dispatch={(action) => dispatch({ ...action, index })}
          key={index}
        />
      ))}
      <input
        type="button"
        value="add config entry"
        onClick={() =>
          dispatch({ type: "addConfigEntry", index: config.entries.length })
        }
      />
      <input type="submit" value="submit" />
    </form>
  );
}

type ConfigEntryAction =
  | { type: "addConfigEntry" }
  | { type: "removeConfigEntry" }
  | { type: "modifyConfigEntry"; configEntry: ConfigEntry };

type ConfigEntryFormProps = {
  configEntry: ConfigEntry;
  dispatch: Dispatch<ConfigEntryAction>;
};

function ConfigEntryForm({
  configEntry,
  dispatch,
}: ConfigEntryFormProps): ReactNode {
  return (
    <div>
      <input
        type="url"
        value={configEntry.imageUrl}
        onChange={(e) =>
          dispatch({
            type: "modifyConfigEntry",
            configEntry: { ...configEntry, imageUrl: e.currentTarget.value },
          })
        }
      />
      <input
        type="number"
        value={configEntry.durationSecs}
        onChange={(e) =>
          dispatch({
            type: "modifyConfigEntry",
            configEntry: {
              ...configEntry,
              durationSecs: e.currentTarget.valueAsNumber,
            },
          })
        }
      />
      <input
        type="button"
        value="add config entry"
        onClick={() => dispatch({ type: "addConfigEntry" })}
      />
      <input
        type="button"
        value="remove config entry"
        onClick={() => dispatch({ type: "removeConfigEntry" })}
      />
    </div>
  );
}

async function fetchConfig(): Promise<Config> {
  const res = await fetch("/config/api");
  const json = await res.json();
  const config = ConfigScheme.parse(json) as Config;
  return config;
}

async function submitConfig(config: Config): Promise<void> {
  ConfigScheme.parse(config);
  await fetch("/config/api", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(config),
  });
}

export default ConfigForm;
