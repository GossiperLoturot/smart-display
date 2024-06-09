import {
  type JSX,
  createEffect,
  createResource,
  createSignal,
  onCleanup,
} from "solid-js";
import "./app.css";
import { Background, Bar, DateTime, Menu, Outline } from "./components";

export const WIDTH = 800;
export const HEIGHT = 480;
export const API_URL = "http://localhost:3000";
export const POLLING_INTERVAL = 250;

export interface Polling {
  dateTime: string;
  imageUrl?: string;
  temperature?: number;
  humidity?: number;
}

export interface ImageIndex {
  durationSecs: number;
  imageUrls: string[];
  imageUrl?: string;
}

export const App = () => {
  const [polling, setPolling] = createSignal<Polling>();
  const [visible, setVisible] = createSignal<boolean>(false);

  createEffect(() => {
    let handle: number | undefined = undefined;

    const fetchState = () => {
      fetch(`${API_URL}/polling`)
        .then((response) => response.json())
        .then((response: Polling) => {
          setPolling(response);
        });
    };

    handle = setInterval(fetchState, POLLING_INTERVAL);

    onCleanup(() => {
      clearInterval(handle);
    });
  });

  const [imageIndex] = createResource(async () => {
    return await fetch(`${API_URL}/image-index`)
      .then((response) => response.json())
      .then((response: ImageIndex) => response);
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
        <Background imageUrl={polling()?.imageUrl} />
        <Outline visible={!visible()} />
        <DateTime
          dateTime={polling()?.dateTime}
          temperature={polling()?.temperature}
          humidity={polling()?.humidity}
        />
        <Bar visible={!visible()} onClick={() => setVisible(true)} />
        <Menu
          visible={visible()}
          onClose={() => setVisible(false)}
          imageIndex={imageIndex()}
        />
      </div>
    </div>
  );
};
