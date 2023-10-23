"use client";

import Image from "next/image";
import { useEffect, useState, ReactNode } from "react";
import { format } from "date-fns";
import { Polling } from "./api/scheme";
import styles from "./component.module.css";

type ClockState = {
  date: string;
  time: string;
  meta: string;
};

export function ClockBlock(): ReactNode {
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
  imageUrl: string;
};

export function BackgroundBlock(): ReactNode {
  const [state, setState] = useState<BackgroundState | undefined>();

  useEffect(() => {
    const handle = setInterval(() => {
      fetchPolling()
        .then((polling) => {
          console.info("successful to fetch polling");
          setState({ imageUrl: polling.imageUrl });
        })
        .catch((reason) => {
          throw reason;
        });
    }, 1000);
    return () => clearInterval(handle);
  }, []);

  if (!state) {
    return (
      <div className={styles["background-container"]}>
        <div className={styles["background"]}>Loading</div>
      </div>
    );
  }

  return (
    <Image
      src={state.imageUrl}
      alt="background"
      className={styles["background-picture"]}
      fill={true}
    />
  );
}

async function fetchPolling(): Promise<Polling> {
  const res = await fetch("/api");
  const json = await res.json();
  return json as Polling;
}
