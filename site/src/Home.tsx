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

    onCleanup(() => {
      clearInterval(handle);
      ws.removeEventListener("open", onOpen);
      ws.removeEventListener("message", onMessage);
      ws.close();
    });
  }, []);

  return (
    <Show when={state()} fallback={<div>Loading</div>}>
      {(state) => (
        <>
          <div class="container-outer">
            <div class="container-inner">
              <ClockComponent dateTime={() => new Date(state().dateTime)} />
              <ProgressComponent dateTime={() => new Date(state().dateTime)} />
            </div>
          </div>
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
    <div class="clock">
      <div class="clock-date">{clock().date}</div>
      <div class="clock-time">{clock().time}</div>
      <div class="clock-meta">{clock().meta}</div>
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
  );
};
