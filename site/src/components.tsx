import { format } from "date-fns";
import * as Icons from "lucide-solid";
import { For, type JSX, Show, createMemo } from "solid-js";
import { API_URL, type ImageIndex } from "./app";

export interface BackgroundProps {
  imageUrl?: string;
}

export const Background = (props: BackgroundProps) => {
  const imageUrl = createMemo(() => props.imageUrl);

  return (
    <Show when={imageUrl()}>
      {(imageUrl) => (
        <img
          src={`${API_URL}/image-get?imageUrl=${imageUrl()}`}
          alt="background"
          class="absolute inset-0 object-cover brightness-75"
        />
      )}
    </Show>
  );
};

export interface DateTimeProps {
  dateTime?: string;
  temperature?: number;
  humidity?: number;
}

export const DateTime = (props: DateTimeProps) => {
  const dateTimeStrings = createMemo(() => {
    if (props.dateTime !== undefined) {
      const now = new Date(props.dateTime);
      const dateString = format(now, "yyyy-MM-dd EEE");
      const timeString = format(now, "HH:mm:ss");
      return { dateString, timeString };
    } else {
      return;
    }
  });

  const sensorStrings = createMemo(() => {
    const temperature = props.temperature?.toFixed(1);
    const humidity = props.humidity?.toFixed(1);
    if (temperature !== undefined && humidity !== undefined) {
      const temperatureString = `${temperature}℃`;
      const humidityString = `${humidity}%RH`;
      return { temperatureString, humidityString };
    } else {
      return;
    }
  });

  return (
    <div class="absolute inset-0 p-[32px] flex">
      <div class="text-white text-center m-auto">
        {/* Date text line */}
        <div class="text-[72px] leading-[1.2] font-bold drop-shadow-[0_4px_8px_rgba(0,0,0,0.25)]">
          <Show
            when={dateTimeStrings()}
            fallback={<div class="invisible">INVISIBLE</div>}
          >
            {(dateTimeStrings) => <>{dateTimeStrings().dateString}</>}
          </Show>
        </div>

        {/* Time text line */}
        <div class="text-[176px] leading-[1.2] drop-shadow-[0_4px_16px_rgba(0,0,0,0.25)]">
          <Show
            when={dateTimeStrings()}
            fallback={<div class="invisible">INVISIBLE</div>}
          >
            {(dateTimeStrings) => <>{dateTimeStrings().timeString}</>}
          </Show>
        </div>

        {/* Sensor text line */}
        <div class="text-[56px] leading-[1.2] font-bold drop-shadow-[0_4px_8px_rgba(0,0,0,0.25)]">
          <Show
            when={sensorStrings()}
            fallback={<div class="invisible">INVISIBLE</div>}
          >
            {(sensorStrings) => (
              <div class="flex justify-evenly">
                <div>{sensorStrings().temperatureString}</div>
                <div>{sensorStrings().humidityString}</div>
              </div>
            )}
          </Show>
        </div>
      </div>
    </div>
  );
};

export interface OutlineProps {
  visible?: boolean;
}

export const Outline = (props: OutlineProps) => {
  return (
    <Show when={props.visible}>
      <div class="absolute inset-0 p-[32px] flex">
        <div class="border border-white flex-grow" />
      </div>
    </Show>
  );
};

export interface BarProps {
  visible?: boolean;
  onClick?: JSX.EventHandlerUnion<HTMLButtonElement, MouseEvent>;
}

export const Bar = (props: BarProps) => {
  return (
    <Show when={props.visible}>
      <button
        class="absolute w-[192px] h-[32px] inset-x-0 bottom-0 mx-auto flex"
        onClick={props.onClick}
      >
        <div class="w-[160px] h-[4px] m-auto bg-white rounded-full drop-shadow-[0_2px_4px_rgba(0,0,0,0.25)]" />
      </button>
    </Show>
  );
};

export interface MenuProps {
  visible?: boolean;
  onClose?: JSX.EventHandlerUnion<
    HTMLButtonElement | HTMLDivElement,
    MouseEvent
  >;
  imageIndex?: ImageIndex;
}

export const Menu = (props: MenuProps) => {
  return (
    <Show when={props.visible}>
      <div class="absolute inset-0 p-[32px]">
        <div class="flex flex-col w-full h-full bg-white rounded-[8px] drop-shadow-[0_4px_16px_rgba(0,0,0,0.25)]">
          {/* Navigation bar */}
          <nav class="flex justify-between items-center gap-[8px] p-[16px]">
            {/* Title */}
            <div class="text-[24px] font-bold">Images</div>

            {/* Search box */}
            <div class="flex items-center gap-[8px] px-[8px] py-[4px] bg-[#EEE] rounded-full">
              <Icons.Link size={16} />
              <input
                class="w-[300px] text-[16px] placeholder-[#444] bg-transparent focus:outline-none"
                type="url"
                placeholder="URLを入力"
              />
              <button>
                <Icons.PlusCircle size={16} />
              </button>
              <button>
                <Icons.CircleCheck size={16} />
              </button>
            </div>

            {/* Timer box */}
            <div class="flex items-center gap-[8px] px-[8px] py-[4px] bg-[#EEE] rounded-full">
              <Icons.Timer size={16} />
              <input
                class="w-[80px] text-[16px] placeholder-[#444] bg-transparent focus:outline-none"
                type="number"
                placeholder="60.0"
              />
              <button>
                <Icons.CircleCheck size={16} />
              </button>
            </div>

            {/* Close button */}
            <button onClick={props.onClose}>
              <Icons.X size={24} />
            </button>
          </nav>

          {/* Image list */}
          <div class="flex-grow overflow-auto">
            <div class="flex flex-col gap-[16px] p-[16px]">
              <For each={props.imageIndex?.imageUrls}>
                {(imageUrl) => <MenuItem imageUrl={imageUrl} />}
              </For>
            </div>
          </div>
        </div>
      </div>
    </Show>
  );
};

export interface MenuItemProps {
  imageUrl: string;
}

const MenuItem = (props: MenuItemProps) => {
  return (
    <div
      class="p-[8px] h-[64px] mx-auto flex rounded-[8px] relative z-[10]"
      style={{
        "background-image": `url(${API_URL}/image-get?imageUrl=${props.imageUrl})`,
      }}
    >
      <div class="absolute inset-0 bg-[rgba(0,0,0,0.25)] z-[-1] rounded-[8px]" />
      <div class="text-white flex items-center gap-[8px] px-[8px] py-[4px] mt-auto">
        {/* URL text line */}
        <div class="w-[500px] truncate">{props.imageUrl}</div>

        {/* Copy */}
        <button>
          <Icons.Copy size={16} />
        </button>

        {/* Apply */}
        <button>
          <Icons.CircleCheck size={16} />
        </button>

        {/* Delete */}
        <button>
          <Icons.MinusCircle size={16} />
        </button>
      </div>
    </div>
  );
};
