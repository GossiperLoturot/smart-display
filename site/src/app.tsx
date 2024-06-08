import { Route, Router } from "@solidjs/router";

import { ConfigPage } from "./config";
import { HomePage } from "./home";

export const API_URL = "http://localhost:3000";

export const App = () => {
  return (
    <Router>
      <Route path="/" component={HomePage} />
      <Route path="/config" component={ConfigPage} />
    </Router>
  );
};
