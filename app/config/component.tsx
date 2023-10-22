"use client";

import { Dispatch, useEffect, useReducer, useState } from "react";
import { Config, ConfigEntry, ConfigEntryScheme, ConfigScheme } from "./scheme";
import { z } from "zod";

class AddEntryAction {
  type: "addEntry";
  index: number;
  constructor(index: number) {
    this.type = "addEntry";
    this.index = index;
  }
}

class RemoveEntryAction {
  type: "removeEntry";
  index: number;
  constructor(index: number) {
    this.type = "removeEntry";
    this.index = index;
  }
}

class UpdateEntryAction {
  type: "updateEntry";
  index: number;
  entry: ConfigEntry;
  constructor(index: number, entry: ConfigEntry) {
    this.type = "updateEntry";
    this.index = index;
    this.entry = entry;
  }
}

class LoadAction {
  type: "load";
  config: Config;
  constructor(config: Config) {
    this.type = "load";
    this.config = config;
  }
}

type Action =
  | AddEntryAction
  | RemoveEntryAction
  | UpdateEntryAction
  | LoadAction;

function reducer(state: Config, dispatch: Action): Config {
  const newState = { ...state };
  switch (dispatch.type) {
    case "addEntry":
      if (0 <= dispatch.index && dispatch.index <= newState.entries.length) {
        newState.entries.splice(dispatch.index, 0, new ConfigEntry());
      }
      return newState;
    case "removeEntry":
      if (0 <= dispatch.index && dispatch.index < newState.entries.length) {
        newState.entries.splice(dispatch.index, 1);
      }
      return newState;
    case "updateEntry":
      if (0 <= dispatch.index && dispatch.index < newState.entries.length) {
        newState.entries[dispatch.index] = dispatch.entry;
      }
      return newState;
    case "load":
      return dispatch.config;
  }
}

export function ConfigForm() {
  const [config, dispatch] = useReducer(reducer, new Config());

  useEffect(() => {
    (async () => {
      const res = await fetch("/config/api");
      const json = await res.json();
      const config = ConfigScheme.parse(json) as Config;
      dispatch(new LoadAction(config));
    })();
  }, []);

  return (
    <form method="dialog" onSubmit={(_) => SubmitForm(config)}>
      {config.entries.map((_, i) => (
        <ConfigEntryForm
          config={config}
          dispatch={dispatch}
          index={i}
          key={i}
        />
      ))}
      <input
        type="button"
        value="add entry"
        onClick={() => dispatch(new AddEntryAction(config.entries.length))}
      />
      <input type="submit" value="submit" />
    </form>
  );
}

type ConfigEntryFormProps = {
  config: Config;
  dispatch: Dispatch<Action>;
  index: number;
};

function ConfigEntryForm({ config, dispatch, index }: ConfigEntryFormProps) {
  const imageUrl = config.entries[index].imageUrl;
  const durationSecs = config.entries[index].durationSecs;

  return (
    <div>
      <input
        type="url"
        value={imageUrl}
        onChange={(e) =>
          dispatch(
            new UpdateEntryAction(
              index,
              new ConfigEntry(e.currentTarget.value, durationSecs),
            ),
          )
        }
      />
      <input
        type="number"
        value={durationSecs}
        onChange={(e) =>
          dispatch(
            new UpdateEntryAction(
              index,
              new ConfigEntry(imageUrl, e.currentTarget.valueAsNumber),
            ),
          )
        }
      />
      <input
        type="button"
        value="add entry"
        onClick={() => dispatch(new AddEntryAction(index))}
      />
      <input
        type="button"
        value="remove entry"
        onClick={() => dispatch(new RemoveEntryAction(index))}
      />
    </div>
  );
}

async function SubmitForm(config: Config) {
  const result = ConfigScheme.safeParse(config);

  if (!result.success) {
    console.log("failed to parse config");
  }

  const res = await fetch("/config/api", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(config),
  });

  console.log(res);
}
