import { For, Show, createResource } from "solid-js";
import { createStore } from "solid-js/store";

import "./config.css";
import { API_URL } from "./app";

interface ImageIndexResponse {
  durationSecs: number;
  imageUrls: string[];
  imageUrl?: string;
}

interface ImageCreateRequest {
  imageUrl: string;
}

interface ImageDeleteRequest {
  imageUrl: string;
}

interface ImageModifyRequest {
  durationSecs?: number;
  imageUrl?: string;
}

export const ConfigPage = () => {
  const [state, { refetch }] = createResource(async () => {
    const response: ImageIndexResponse = await fetch(
      `${API_URL}/image-index`,
    ).then((response) => response.json());
    return response;
  });

  const [pushForm, setPushForm] = createStore({ imageUrl: "" });
  const [patchForm, setPatchForm] = createStore({ durationSecs: "" });

  const onCreate = async (request: ImageCreateRequest) => {
    await fetch(`${API_URL}/image-create`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  const onDelete = async (request: ImageDeleteRequest) => {
    await fetch(`${API_URL}/image-delete`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(request),
    });
    await refetch();
  };

  const onModify = async (request: ImageModifyRequest) => {
    await fetch(`${API_URL}/image-modify`, {
      method: "POST",
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
                  onModify({
                    durationSecs: Number.parseFloat(patchForm.durationSecs),
                  })
                }
              >
                submit
              </button>
            </div>
            <div class="head-bottom">
              <div class="item-img-container">
                <img
                  src={`${API_URL}/image-get?imageUrl=${state().imageUrl}`}
                  width="100px"
                  height="100px"
                  class="item-img"
                />
              </div>
              <div class="item-url-container">
                <div class="item-url">{state().imageUrl}</div>
              </div>
            </div>
          </div>
          <div class="item">
            <div class="item-img-container">
              <img
                src={`${API_URL}/image-get?imageUrl=${pushForm.imageUrl}`}
                width="100px"
                height="100px"
                class="item-img"
              />
            </div>
            <div class="item-url-input-container">
              <textarea
                class="item-url-input"
                placeholder="https://example.com/example.png"
                value={pushForm.imageUrl}
                onInput={(e) =>
                  setPushForm({ imageUrl: e.currentTarget.value })
                }
              />
            </div>
            <div class="item-act-container">
              <button
                class="item-act"
                onClick={() => onCreate({ imageUrl: pushForm.imageUrl })}
              >
                +
              </button>
              <button
                class="item-act"
                onClick={() => onModify({ imageUrl: pushForm.imageUrl })}
              >
                *
              </button>
            </div>
          </div>
          <For each={state().imageUrls}>
            {(url, i) => (
              <div class="item">
                <div class="item-img-container">
                  <img
                    src={`${API_URL}/image-get?imageUrl=${state().imageUrls[i()]}`}
                    width="100px"
                    height="100px"
                    class="item-img"
                  />
                </div>
                <div class="item-url-container">
                  <div class="item-url">{url}</div>
                </div>
                <div class="item-act-container">
                  <button
                    class="item-act"
                    onClick={() => onDelete({ imageUrl: url })}
                  >
                    -
                  </button>
                  <button
                    class="item-act"
                    onClick={() => onModify({ imageUrl: url })}
                  >
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
