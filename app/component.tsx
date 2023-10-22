"use client";

import { useEffect, useState } from "react";
import { Config, ConfigScheme } from "./config/scheme";
import Image from "next/image";

export function Clock() {
  const [clock, setClock] = useState<string | undefined>();

  useEffect(() => {
    setInterval(() => {
      const date = new Date().toString();
      setClock(date);
    }, 100);
  }, []);

  return <div>{clock}</div>;
}

export function Background() {
  const [config, setConfig] = useState<Config | undefined>();
  const [url, setUrl] = useState<string | undefined>();

  useEffect(() => {
    (async () => {
      const res = await fetch("/config/api");
      const json = await res.json();
      const config = ConfigScheme.parse(json) as Config;
      setConfig(config);

      setUrl(config.entries[0].imageUrl);
    })();
  }, []);

  if (url == undefined) {
    return <div>loading</div>;
  }

  return <Image src={url} alt="" width={720} height={420} />;
}
