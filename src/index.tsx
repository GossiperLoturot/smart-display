/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Routes, Route } from "@solidjs/router";
import { HomePage, PicturePage } from "./App";

export const HTTP_API =
  import.meta.env.VITE_HTTP_API || `http://${window.location.host}/api`;
export const WS_API =
  import.meta.env.VITE_WS_API || `ws://${window.location.host}/api`;
console.log({ HTTP_API, WS_API });

const root = document.getElementById("root");

render(
  () => (
    <Router>
      <Routes>
        <Route path="/" component={HomePage} />
        <Route path="/pic" component={PicturePage} />
      </Routes>
    </Router>
  ),
  root!,
);
