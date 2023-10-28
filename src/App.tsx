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
import "./App.css";

type Polling = {
  dateTime: string;
  url: string;
};

export const HomePage = () => {
  const [polling, setPolling] = createSignal<Polling | undefined>();

  createEffect(() => {
    let handle: number | undefined = undefined;
    const onOpen = () => {
      handle = setInterval(() => {
        ws.send("");
      }, 250);
    };

    const onMessage = (event: MessageEvent<string>) => {
      const polling = JSON.parse(event.data);
      setPolling(polling);
    };

    const ws = new WebSocket("ws://localhost:3000/polling");
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
    <Show when={polling()} fallback={<div>Loading</div>}>
      {(polling) => (
        <>
          <ClockComponent dateTime={() => new Date(polling().dateTime)} />
          <img src={polling().url} class="bg" />
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

type PicListOut = { durationSecs: number; urls: string[]; url: string };
type PicPushIn = { url: string };
type PicPopIn = { url: string };
type PicPatchIn = { url?: string; durationSecs?: number };

export const PicturePage = () => {
  const [picList, { refetch }] = createResource(async () => {
    const response = await fetch("http://localhost:3000/pic");
    const output: PicListOut = await response.json();
    return output;
  });
  const [pushForm, setPushForm] = createStore({ url: "" });
  const [patchForm, setPatchForm] = createStore({ durationSecs: "" });

  const onPush = async (input: PicPushIn) => {
    await fetch("http://localhost:3000/pic", {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    });
    await refetch();
  };

  const onPop = async (input: PicPopIn) => {
    await fetch("http://localhost:3000/pic", {
      method: "DELETE",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    });
    await refetch();
  };

  const onPatch = async (input: PicPatchIn) => {
    await fetch("http://localhost:3000/pic", {
      method: "PATCH",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(input),
    });
    await refetch();
  };

  return (
    <Show when={picList()} fallback={<div>Loading</div>}>
      {(picList) => (
        <>
          <div class="title">Pictures</div>
          <div class="head">
            <div class="head-top">
              <div class="head-top-label">
                Duration: {picList().durationSecs} secs
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
                  onPatch({ durationSecs: parseFloat(patchForm.durationSecs) })
                }
              >
                submit
              </button>
            </div>
            <div class="head-bottom">
              <div class="item-img-container">
                <img
                  src={picList().url}
                  width="100px"
                  height="100px"
                  class="item-img"
                />
              </div>
              <div class="item-url-container">
                <div class="item-url">{picList().url}</div>
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
                onClick={() => onPush({ url: pushForm.url })}
              >
                +
              </button>
              <button
                class="item-act"
                onClick={() => onPatch({ url: pushForm.url })}
              >
                *
              </button>
            </div>
          </div>
          <For each={picList().urls}>
            {(url) => (
              <div class="item">
                <div class="item-img-container">
                  <img
                    src={url}
                    width="100px"
                    height="100px"
                    class="item-img"
                  />
                </div>
                <div class="item-url-container">
                  <div class="item-url">{url}</div>
                </div>
                <div class="item-act-container">
                  <button class="item-act" onClick={() => onPop({ url })}>
                    -
                  </button>
                  <button class="item-act" onClick={() => onPatch({ url })}>
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
