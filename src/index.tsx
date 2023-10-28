/* @refresh reload */
import { render } from "solid-js/web";
import { Router, Routes, Route } from "@solidjs/router";
import { HomePage, PicturePage } from "./App";

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
