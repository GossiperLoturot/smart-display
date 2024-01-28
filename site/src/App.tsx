import {
  endOfDay,
  endOfMonth,
  endOfYear,
  format,
  startOfDay,
  startOfMonth,
  startOfYear,
} from "date-fns";
import {
  For,
  Show,
  createEffect,
  createMemo,
  createResource,
  createSignal,
} from "solid-js";
import { createStore } from "solid-js/store";
import "./App.css";
import { mock } from "./mock";

type HomePageState = { dateTime: string; url?: string };
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
      setState(response);
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
          <ProgressComponent dateTime={() => new Date(state().dateTime)} />
          <Show when={state().url}>
            {(url) => <img src={url()} class="bg" />}
          </Show>
        </>
      )}
    </Show>
  );
};

const ClockComponent = ({ dateTime }: { dateTime: () => Date }) => {
  const clock = createMemo(() => {
    const now = dateTime();
    return {
      date: format(now, "yyyy/MM/dd eeee, BBBB"),
      time: format(now, "HH:mm:ss"),
      meta: format(now, "QQQQ, OOOO"),
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

const ProgressComponent = ({ dateTime }: { dateTime: () => Date }) => {
  function threshold(x: number, start: number, end: number) {
    return (x - start) / (end - start);
  }

  const progress = createMemo(() => {
    const now = dateTime();
    const day = threshold(
      now.valueOf(),
      startOfDay(now).valueOf(),
      endOfDay(now).valueOf(),
    );
    const month = threshold(
      now.valueOf(),
      startOfMonth(now).valueOf(),
      endOfMonth(now).valueOf(),
    );
    const year = threshold(
      now.valueOf(),
      startOfYear(now).valueOf(),
      endOfYear(now).valueOf(),
    );
    return { day, month, year };
  });

  return (
    <div class="progress-container">
      <div class="progress">
        <div class="progress-day">
          <div class="progress-label">Day</div>
          <div class="progress-value">{`${(progress().day * 100.0).toFixed(
            1,
          )}%`}</div>
          <div class="progress-barouter">
            <div
              class="progress-barinner"
              style={`width:${progress().day * 100.0}%`}
            />
          </div>
        </div>
        <div class="progress-month">
          <div class="progress-label">Month</div>
          <div class="progress-value">{`${(progress().month * 100.0).toFixed(
            1,
          )}%`}</div>
          <div class="progress-barouter">
            <div
              class="progress-barinner"
              style={`width:${progress().month * 100.0}%`}
            />
          </div>
        </div>
        <div class="progress-year">
          <div class="progress-label">Year</div>
          <div class="progress-value">{`${(progress().year * 100.0).toFixed(
            1,
          )}%`}</div>
          <div class="progress-barouter">
            <div
              class="progress-barinner"
              style={`width:${progress().year * 100.0}%`}
            />
          </div>
        </div>
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

export const PicturePage = () => {
  const [state, { refetch }] = createResource(async () => {
    const response: PictureIndexResponse = await fetch(
      `${mock.apiUrl}/config`,
    ).then((response) => response.json());
    return response;
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
                  src={state().url}
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
                    src={state().urls[i()]}
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
