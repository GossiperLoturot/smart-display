"use client";

import { useEffect, useState, ReactNode } from "react";
import { Config, ConfigScheme } from "./config/scheme";
import styles from "./component.module.css";
import { format } from "date-fns";

type ClockState = {
  date: string;
  time: string;
  meta: string;
};

export function Clock(): ReactNode {
  const [state, setState] = useState<ClockState | undefined>();

  useEffect(() => {
    const handle = setInterval(() => {
      const now = new Date();
      setState({
        date: format(now, "yyyy/MM/dd eeee, BBBB"),
        time: format(now, "HH:mm:ss"),
        meta: format(now, "QQQQ, OOOO"),
      });
    }, 100);
    return () => clearInterval(handle);
  }, []);

  if (!state) {
    return (
      <div className={styles["clock-container"]}>
        <div className={styles["clock"]}>Loading</div>
      </div>
    );
  }

  return (
    <div className={styles["clock-container"]}>
      <div className={styles["clock"]}>
        <div className={styles["clock-date"]}>{state.date}</div>
        <div className={styles["clock-time"]}>{state.time}</div>
        <div className={styles["clock-meta"]}>{state.meta}</div>
      </div>
    </div>
  );
}

type BackgroundState = {
  url: string;
};

export function Background(): ReactNode {
  const [state, setState] = useState<BackgroundState | undefined>();

  useEffect(() => {
    fetchConfig()
      .then((config) => {
        console.info("successful to fetch config");
        setState({ url: config.entries[1].imageUrl });
      })
      .catch((reason) => {
        throw reason;
      });
  }, []);

  if (!state) {
    return (
      <div className={styles["background-container"]}>
        <div className={styles["background"]}>Loading</div>
      </div>
    );
  }

  return (
    <img
      src={state.url}
      alt="background"
      className={styles["background-picture"]}
    />
  );
}

async function fetchConfig(): Promise<Config> {
  const res = await fetch("/config/api");
  const json = await res.json();
  const config = ConfigScheme.parse(json) as Config;
  return config;
}
