import { format } from "date-fns";
import {
  For,
  Show,
  createEffect,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import { createStore } from "solid-js/store";
import { mock } from "./mock";
import "./App.css";

type HomePageState = { dateTime: string; url?: string; cachedUrl?: string };
type PollingResponse = { dateTime: string; url?: string };

export const HomePage = () => {
  const [state, setState] = createSignal<HomePageState | undefined>();

  createEffect(() => {
    let handle: number | undefined = undefined;
    const onOpen = () => {
      handle = setInterval(() => {
        ws.send("");
      }, 250);
    };

    const onMessage = async (event: MessageEvent<string>) => {
      const response: PollingResponse = JSON.parse(event.data);
      if (response.url != undefined) {
        const cachedUrl = await getCachePath(response.url);
        setState({ ...response, cachedUrl });
      } else {
        setState(response);
      }
    };

    const ws = new WebSocket(`${mock.wsUrl}/polling`);
    ws.addEventListener("open", onOpen);
    ws.addEventListener("message", onMessage);

    return () => {
      clearInterval(handle);
      ws.removeEventListener("open", onOpen);
      ws.removeEventListener("message", onMessage);
      ws.close();
    };
  }, []);

  return (
    <Show when={state()} fallback={<div>Loading</div>}>
      {(state) => (
        <>
          <ClockComponent dateTime={() => new Date(state().dateTime)} />
          <Show when={state().cachedUrl}>
            {(cachedUrl) => <img src={cachedUrl()} class="bg" />}
          </Show>
        </>
      )}
    </Show>
  );
};

const ClockComponent = ({ dateTime }: { dateTime: () => Date }) => {
  const clock = createMemo(() => {
    const date = new Date(dateTime());
    return {
      date: format(date, "yyyy/MM/dd eeee, BBBB"),
      time: format(date, "HH:mm:ss"),
      meta: format(date, "QQQQ, OOOO"),
    };
  });

  return (
    <div class="clock-container">
      <div class="clock">
        <div class="clock-date">{clock().date}</div>
        <div class="clock-time">{clock().time}</div>
        <div class="clock-meta">{clock().meta}</div>
      </div>
    </div>
  );
};

type PictureIndexResponse = {
  durationSecs: number;
  urls: string[];
  url?: string;
};
type PictureCreateRequest = { url: string };
type PictureDeleteRequest = { url: string };
type PictureApplyRequest = { url?: string; durationSecs?: number };
type PictureCacheRequest = { url: string };
type PictureCacheResponse = { id: string };

export const PicturePage = () => {
  const [state, { refetch }] = createResource(async () => {
    const response: PictureIndexResponse = await fetch(
      `${mock.apiUrl}/config`,
    ).then((response) => response.json());

    let cachedUrl = undefined;
    if (response.url != undefined) {
      cachedUrl = await getCachePath(response.url);
    }

    const cachedUrls = new Array(response.urls.length);
    for (let i = 0; i < response.urls.length; i++) {
      cachedUrls[i] = await getCachePath(response.urls[i]);
    }

    return { ...response, cachedUrl, cachedUrls };
  });
  const [pushForm, setPushForm] = createStore({ url: "" });
  const [patchForm, setPatchForm] = createStore({ durationSecs: "" });

  const onCreate = async (request: PictureCreateRequest) => {
    await fetch(`${mock.apiUrl}/config`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  const onDelete = async (request: PictureDeleteRequest) => {
    await fetch(`${mock.apiUrl}/config`, {
      method: "DELETE",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  const onApply = async (request: PictureApplyRequest) => {
    await fetch(`${mock.apiUrl}/config`, {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  return (
    <Show when={state()} fallback={<div>Loading</div>}>
      {(state) => (
        <>
          <div class="title">Pictures</div>
          <div class="head">
            <div class="head-top">
              <div class="head-top-label">
                Duration: {state().durationSecs} secs
              </div>
              <input
                type="number"
                class="head-top-input"
                placeholder="60.0"
                value={patchForm.durationSecs}
                onInput={(e) =>
                  setPatchForm({ durationSecs: e.currentTarget.value })
                }
              />
              <button
                class="head-top-button"
                onClick={() =>
                  onApply({ durationSecs: parseFloat(patchForm.durationSecs) })
                }
              >
                submit
              </button>
            </div>
            <div class="head-bottom">
              <div class="item-img-container">
                <img
                  src={state().cachedUrl}
                  width="100px"
                  height="100px"
                  class="item-img"
                />
              </div>
              <div class="item-url-container">
                <div class="item-url">{state().url}</div>
              </div>
            </div>
          </div>
          <div class="item">
            <div class="item-img-container">
              <img
                src={pushForm.url}
                width="100px"
                height="100px"
                class="item-img"
              />
            </div>
            <div class="item-url-input-container">
              <textarea
                class="item-url-input"
                placeholder="https://example.com/example.png"
                value={pushForm.url}
                onInput={(e) => setPushForm({ url: e.currentTarget.value })}
              />
            </div>
            <div class="item-act-container">
              <button
                class="item-act"
                onClick={() => onCreate({ url: pushForm.url })}
              >
                +
              </button>
              <button
                class="item-act"
                onClick={() => onApply({ url: pushForm.url })}
              >
                *
              </button>
            </div>
          </div>
          <For each={state().urls}>
            {(url, i) => (
              <div class="item">
                <div class="item-img-container">
                  <img
                    src={state().cachedUrls[i()]}
                    width="100px"
                    height="100px"
                    class="item-img"
                  />
                </div>
                <div class="item-url-container">
                  <div class="item-url">{url}</div>
                </div>
                <div class="item-act-container">
                  <button class="item-act" onClick={() => onDelete({ url })}>
                    -
                  </button>
                  <button class="item-act" onClick={() => onApply({ url })}>
                    *
                  </button>
                </div>
              </div>
            )}
          </For>
        </>
      )}
    </Show>
  );
};

async function getCachePath(url: string): Promise<string> {
  const request: PictureCacheRequest = { url };
  const response: PictureCacheResponse = await fetch(`${mock.apiUrl}/cache`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  }).then((response) => response.json());
  return `${mock.cacheUrl}/${response.id}`;
}
