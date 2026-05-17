import {
  createEffect,
  createResource,
  createSignal,
  type JSX,
  Match,
  onCleanup,
  Show,
  Switch,
} from "solid-js";
import "./app.css";
import { Background, Bar, DateTime, Menu, Outline } from "./components";

export const WIDTH = import.meta.env.VITE_HEIGHT || 780;
export const HEIGHT = import.meta.env.VITE_WIDTH || 460;
export const API_URL = import.meta.env.VITE_API_URL || window.location.origin;
export const POLLING_INTERVAL = import.meta.env.VITE_POLLING_INTERVAL || 250;

export interface PollingResponse {
  dateTime: string;
  currentImageId: string | undefined;
}

export interface ImageIndexResponse {
  durationSecs: number;
  imageIds: string[];
  currentImageId: string | undefined;
}

export interface ImageModifyRequest {
  durationSecs: number | undefined;
  currentImageId: string | undefined;
}

export interface ImageCreateRequest {
  image: File;
}

export interface ImageDeleteRequest {
  imageId: string;
}

export const App = () => {
  const [polling, setPolling] = createSignal<PollingResponse>();
  const [holdMenu, setHoldMenu] = createSignal<boolean>(false);

  createEffect(() => {
    let handle: number | undefined;

    const fetchState = () => {
      fetch(`${API_URL}/polling`)
        .then((response) => response.json())
        .then((response: PollingResponse) => {
          setPolling(response);
        });
    };

    handle = setInterval(fetchState, POLLING_INTERVAL);

    onCleanup(() => {
      clearInterval(handle);
    });
  });

  const [imageIndex, { refetch }] = createResource(async () => {
    return await fetch(`${API_URL}/image-index`)
      .then((response) => response.json())
      .then((response: ImageIndexResponse) => response);
  });

  const onModify = async (request: ImageModifyRequest) => {
    await fetch(`${API_URL}/image-modify`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  const onCreate = async (request: ImageCreateRequest) => {
    const form = new FormData();
    form.append("image", request.image);
    await fetch(`${API_URL}/image-create`, {
      method: "POST",
      body: form,
    });
    await refetch();
  };

  const onDelete = async (request: ImageDeleteRequest) => {
    await fetch(`${API_URL}/image-delete`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  const currentImageIdState = () => polling()?.currentImageId;
  const dateTimeState = () => {
    const pollingValue = polling();
    if (pollingValue?.dateTime) {
      return {
        dateTime: pollingValue.dateTime,
      };
    }
    return;
  };

  const containerStyle: JSX.CSSProperties = {
    width: `${WIDTH}px`,
    "min-width": `${WIDTH}px`,
    height: `${HEIGHT}px`,
    "min-height": `${HEIGHT}px`,
  };

  return (
    <div class="w-screen h-screen flex">
      <div class="m-auto relative" style={containerStyle}>
        <Show when={currentImageIdState()}>
          {(currentImageId) => <Background imageId={currentImageId()} />}
        </Show>
        <Show when={dateTimeState()}>
          {(dateTime) => <DateTime dateTime={dateTime().dateTime} />}
        </Show>
        <Switch>
          <Match when={!holdMenu()}>
            <Outline />
            <Bar onClick={() => setHoldMenu(true)} />
          </Match>
          <Match when={holdMenu()}>
            <Show when={imageIndex()}>
              {(imageIndex) => (
                <Menu
                  onClose={() => setHoldMenu(false)}
                  imageIndex={imageIndex()}
                  onModify={onModify}
                  onCreate={onCreate}
                  onDelete={onDelete}
                />
              )}
            </Show>
          </Match>
        </Switch>
      </div>
    </div>
  );
};
