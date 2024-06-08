import { format } from "date-fns";
import { Show, createMemo } from "solid-js";
import { API_URL } from "./app";

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
          class="absolute top-0 right-0 w-full h-full object-cover brightness-75"
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
  const nowMemo = createMemo(() => {
    if (props.dateTime !== undefined) {
      const now = new Date(props.dateTime);
      const dateString = format(now, "yyyy-MM-dd EEE");
      const timeString = format(now, "HH:mm:ss");
      return { dateString, timeString };
    } else {
      return;
    }
  });

  const sensorMemo = createMemo(() => {
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
    <div class="absolute top-0 right-0 w-full h-full p-[32px] flex items-center justify-center">
      <div class="text-white text-center">
        <div class="text-[72px] font-bold leading-[86px] drop-shadow-[0_4px_8px_rgba(0,0,0,0.25)]">
          {nowMemo()?.dateString}
        </div>
        <div class="text-[176px] leading-[212px] drop-shadow-[0_4px_16px_rgba(0,0,0,0.25)]">
          {nowMemo()?.timeString}
        </div>
        <div class="text-[56px] font-bold leading-[68px] drop-shadow-[0_4px_8px_rgba(0,0,0,0.25)] flex justify-evenly h-[68px]">
          <div>{sensorMemo()?.temperatureString}</div>
          <div>{sensorMemo()?.humidityString}</div>
        </div>
      </div>
    </div>
  );
};

export const Outline = () => {
  return (
    <div class="absolute top-0 right-0 w-full h-full p-[32px] flex">
      <div class="border border-white flex-grow" />
    </div>
  );
};

export const Bar = () => {
  return (
    <div class="absolute bottom-0 right-0 w-full h-[32px] flex items-center justify-center">
      <div class="w-[192px] h-[32px] flex items-center justify-center">
        <div class="w-[160px] h-[4px] bg-white rounded-full drop-shadow-[0_2px_4px_rgba(0,0,0,0.25)]" />
      </div>
    </div>
  );
};
