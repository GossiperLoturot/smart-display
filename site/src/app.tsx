import { type JSX, createEffect, createSignal, onCleanup } from "solid-js";
import "./app.css";
import { Background, Bar, DateTime, Outline } from "./components";

export const WIDTH = 800;
export const HEIGHT = 480;
export const API_URL = "http://localhost:3000";
export const POLLING_INTERVAL = 250;

export interface State {
  dateTime: string;
  imageUrl?: string;
  temperature?: number;
  humidity?: number;
}

export const App = () => {
  const [state, setState] = createSignal<State>();

  createEffect(() => {
    let handle: number | undefined = undefined;

    const fetchState = () => {
      fetch(`${API_URL}/polling`)
        .then((response) => response.json())
        .then((response: State) => {
          setState(response);
        });
    };

    handle = setInterval(fetchState, POLLING_INTERVAL);

    onCleanup(() => {
      clearInterval(handle);
    });
  });

  const containerStyle: JSX.CSSProperties = {
    width: `${WIDTH}px`,
    "min-width": `${WIDTH}px`,
    height: `${HEIGHT}px`,
    "min-height": `${HEIGHT}px`,
  };

  return (
    <div class="w-screen h-screen flex">
      <div class="m-auto relative" style={containerStyle}>
        <Background imageUrl={state()?.imageUrl} />
        <Outline />
        <DateTime
          dateTime={state()?.dateTime}
          temperature={state()?.temperature}
          humidity={state()?.humidity}
        />
        <Bar />
      </div>
    </div>
  );
};
