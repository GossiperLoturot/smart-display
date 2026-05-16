import { format } from "date-fns";
import * as Icons from "lucide-solid";
import { createMemo, createSignal, For, type JSX } from "solid-js";
import {
  API_URL,
  type ImageCreateRequest,
  type ImageDeleteRequest,
  type ImageIndexResponse,
  type ImageModifyRequest,
} from "./app";

export interface BackgroundProps {
  imageKey: string;
}

export const Background = (props: BackgroundProps) => {
  const imageKey = createMemo(() => props.imageKey);

  return (
    <img
      src={`${API_URL}/image-get?imageKey=${imageKey()}`}
      alt="background"
      class="absolute inset-0 object-cover brightness-75"
    />
  );
};

export interface DateTimeProps {
  dateTime: string;
}

export const DateTime = (props: DateTimeProps) => {
  const dateTimeStrings = createMemo(() => {
    const now = new Date(props.dateTime);
    const dateString = format(now, "yyyy-MM-dd EEE");
    const timeString = format(now, "HH:mm:ss");
    return { dateString, timeString };
  });

  return (
    <div class="absolute inset-0 p-[32px] flex">
      <div class="text-white text-center m-auto">
        {/* Date text line */}
        <div class="text-[72px] leading-[1.2] font-bold drop-shadow-[0_4px_8px_rgba(0,0,0,0.25)]">
          {dateTimeStrings().dateString}
        </div>

        {/* Time text line */}
        <div class="text-[176px] leading-[1.2] drop-shadow-[0_4px_16px_rgba(0,0,0,0.25)]">
          {dateTimeStrings().timeString}
        </div>
      </div>
    </div>
  );
};

export const Outline = () => {
  return (
    <div class="absolute inset-0 p-[32px] flex">
      <div class="border border-white flex-grow" />
    </div>
  );
};

export interface BarProps {
  onClick: JSX.EventHandlerUnion<HTMLButtonElement, MouseEvent>;
}

export const Bar = (props: BarProps) => {
  return (
    <button
      type="button"
      class="absolute w-[192px] h-[32px] inset-x-0 bottom-0 mx-auto flex"
      onClick={props.onClick}
    >
      <div class="w-[160px] h-[4px] m-auto bg-white rounded-full drop-shadow-[0_2px_4px_rgba(0,0,0,0.25)]" />
    </button>
  );
};

export interface MenuProps {
  onClose: JSX.EventHandlerUnion<
    HTMLButtonElement | HTMLDivElement,
    MouseEvent
  >;
  imageIndex: ImageIndexResponse;
  onModify: (request: ImageModifyRequest) => Promise<void>;
  onCreate: (request: ImageCreateRequest) => Promise<void>;
  onDelete: (request: ImageDeleteRequest) => Promise<void>;
}

export const Menu = (props: MenuProps) => {
  const [uploadFile, setUploadFile] = createSignal<File | null>(null);
  const [timer, setTimer] = createSignal("");

  const onCreate = () => {
    const file = uploadFile();
    if (file) {
      props.onCreate({ image: file }).then(() => {
        setUploadFile(null);
      });
    }
  };

  const onModifyTimer = () => {
    const durationSecs = Number.parseFloat(timer());
    return props.onModify({ imageKey: undefined, durationSecs });
  };

  return (
    <div class="absolute inset-0 p-[32px]">
      <div class="flex flex-col w-full h-full bg-white rounded-[8px] drop-shadow-[0_4px_16px_rgba(0,0,0,0.25)]">
        {/* Navigation bar */}
        <nav class="flex justify-between items-center gap-[8px] p-[16px]">
          {/* Title */}
          <div class="text-[24px] font-bold">Images</div>

          {/* Upload box */}
          <div class="flex items-center gap-[8px] px-[8px] py-[4px] bg-[#EEE] rounded-full">
            <Icons.ImagePlus size={16} />
            <input
              type="file"
              accept="image/*"
              class="w-[300px] text-[16px] bg-transparent focus:outline-none"
              onInput={(event) => {
                const file = event.currentTarget.files?.[0];
                if (file) setUploadFile(file);
              }}
            />
            <button
              type="button"
              class="active:scale-125 transition"
              onClick={onCreate}
              disabled={!uploadFile()}
            >
              <Icons.Upload size={16} />
            </button>
          </div>

          {/* Timer box */}
          <div class="flex items-center gap-[8px] px-[8px] py-[4px] bg-[#EEE] rounded-full">
            <Icons.Timer size={16} />
            <input
              type="number"
              class="w-[80px] text-[16px] placeholder-[#888] bg-transparent focus:outline-none"
              placeholder={props.imageIndex.durationSecs.toFixed(1)}
              value={timer()}
              onInput={(event) => setTimer(event.currentTarget.value)}
            />
            <button
              type="button"
              class="active:scale-125 transition"
              onClick={onModifyTimer}
            >
              <Icons.CircleCheck size={16} />
            </button>
          </div>

          {/* Close button */}
          <button
            type="button"
            class="active:scale-125 transition"
            onClick={props.onClose}
          >
            <Icons.X size={24} />
          </button>
        </nav>

        {/* Image list */}
        <div class="flex-grow overflow-auto">
          <div class="flex flex-col gap-[16px] p-[16px]">
            <For each={props.imageIndex.imageKeys}>
              {(imageKey) => (
                <MenuItem
                  imageKey={imageKey}
                  onModify={props.onModify}
                  onDelete={props.onDelete}
                />
              )}
            </For>
          </div>
        </div>
      </div>
    </div>
  );
};

export interface MenuItemProps {
  imageKey: string;
  onModify: (request: ImageModifyRequest) => Promise<void>;
  onDelete: (request: ImageDeleteRequest) => Promise<void>;
}

const MenuItem = (props: MenuItemProps) => {
  const imageKey = createMemo(() => props.imageKey);

  const onCopy = () => {
    navigator.clipboard.writeText(imageKey());
  };

  const onModifyImage = () => {
    props.onModify({ imageKey: imageKey(), durationSecs: undefined });
  };

  const onDelete = () => {
    props.onDelete({ imageKey: imageKey() });
  };

  return (
    <div
      class="p-[8px] h-[64px] mx-auto flex rounded-[8px] relative z-[10]"
      style={{
        "background-image": `url(${API_URL}/image-get?imageKey=${imageKey()})`,
      }}
    >
      <div class="absolute inset-0 bg-[rgba(0,0,0,0.25)] z-[-1] rounded-[8px]" />
      <div class="text-white flex items-center gap-[8px] px-[8px] py-[4px] mt-auto">
        {/* URL text line */}
        <div class="w-[500px] truncate">{imageKey()}</div>

        {/* Copy */}
        <button
          type="button"
          class="active:scale-125 transition"
          onClick={onCopy}
        >
          <Icons.Copy size={16} />
        </button>

        {/* Apply */}
        <button
          type="button"
          class="active:scale-125 transition"
          onClick={onModifyImage}
        >
          <Icons.CircleCheck size={16} />
        </button>

        {/* Delete */}
        <button
          type="button"
          class="active:scale-125 transition"
          onClick={onDelete}
        >
          <Icons.CircleMinus size={16} />
        </button>
      </div>
    </div>
  );
};
