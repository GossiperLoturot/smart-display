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
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { mock } from "./mock";
import "./Home.css";

interface HomePageState {
  dateTime: string;
  url?: string;
  temperature?: number;
  humidity?: number;
}

interface PollingResponse {
  dateTime: string;
  url?: string;
  temperature?: number;
  humidity?: number;
}

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

    onCleanup(() => {
      clearInterval(handle);
      ws.removeEventListener("open", onOpen);
      ws.removeEventListener("message", onMessage);
      ws.close();
    });
  }, []);

  return (
    <Show when={state()}>
      {(state) => (
        <>
          <InnerComponent state={state()} />
          <BgComponent state={state()} />
        </>
      )}
    </Show>
  );
};

const InnerComponent = (props: { state: HomePageState }) => {
  const memo = createMemo(() => {
    const now = new Date(props.state.dateTime);

    const date = format(now, "yyyy/MM/dd eeee, BBBB");
    const time = format(now, "HH:mm:ss");
    let meta = format(now, "QQQQ, OOOO");

    if (props.state.temperature) {
      meta = meta.concat(`, ${props.state.temperature}°C`);
    }
    if (props.state.humidity) {
      meta = meta.concat(`, ${props.state.humidity}%RH`);
    }

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

    return { date, time, meta, day, month, year };
  });

  return (
    <div class="container-outer">
      <div class="container-inner">
        <div class="clock-date">{memo().date}</div>
        <div class="clock-time">{memo().time}</div>
        <div class="clock-meta">{memo().meta}</div>

        <div class="gap"></div>

        <BarComponent label="Day" value={memo().day} />
        <BarComponent label="Month" value={memo().month} />
        <BarComponent label="Year" value={memo().year} />
      </div>
    </div>
  );
};

const BarComponent = (props: { label: string; value: number }) => {
  return (
    <div class="progress">
      <div class="progress-label">{props.label}</div>
      <div class="progress-value">{`${(props.value * 100.0).toFixed(1)}%`}</div>
      <div class="progress-bar-outer">
        <div
          class="progress-bar-inner"
          style={`width:${props.value * 100.0}%`}
        ></div>
      </div>
    </div>
  );
};

const BgComponent = (props: { state: HomePageState }) => {
  return (
    <Show when={props.state.url}>
      {(url) => <img src={`${mock.apiUrl}/buffer?url=${url()}`} class="bg" />}
    </Show>
  );
};

function threshold(x: number, start: number, end: number) {
  return (x - start) / (end - start);
}
