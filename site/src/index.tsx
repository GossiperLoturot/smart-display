import { Route, Router } from "@solidjs/router";
/* @refresh reload */
import { render } from "solid-js/web";
import { HomePage, PicturePage } from "./App";

const root = document.getElementById("root");

render(
  () => (
    <Router>
      <Route path="/" component={HomePage} />
      <Route path="/config" component={PicturePage} />
    </Router>
  ),
  root!,
);
