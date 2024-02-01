/* @refresh reload */

import { render } from "solid-js/web";
import { Route, Router } from "@solidjs/router";
import { HomePage } from "./Home";
import { ConfigPage } from "./Config";

const root = document.getElementById("root");

render(
  () => (
    <Router>
      <Route path="/" component={HomePage} />
      <Route path="/config" component={ConfigPage} />
    </Router>
  ),
  root!,
);
