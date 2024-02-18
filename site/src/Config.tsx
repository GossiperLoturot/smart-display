import { For, Show, createResource } from "solid-js";
import { createStore } from "solid-js/store";
import { mock } from "./mock";
import "./Config.css";

interface PictureIndexResponse {
  durationSecs: number;
  urls: string[];
  url?: string;
}

interface PictureCreateRequest {
  url: string;
}

interface PictureDeleteRequest {
  url: string;
}

interface PictureApplyRequest {
  url?: string;
  durationSecs?: number;
}

export const ConfigPage = () => {
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
                  src={`${mock.apiUrl}/buffer?url=${state().url}`}
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
                src={`${mock.apiUrl}/buffer?url=${pushForm.url}`}
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
                    src={`${mock.apiUrl}/buffer?url=${state().urls[i()]}`}
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
