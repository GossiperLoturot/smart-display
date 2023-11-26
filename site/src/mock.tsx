export type Mock = {
  apiUrl: String;
  wsUrl: String;
  cacheUrl: String;
};

function initMock(): Mock {
  if (process.env.NODE_ENV !== "development") {
    const apiUrl = `http://${window.location.host}/api`;
    const wsUrl = `ws://${window.location.host}/api`;
    const cacheUrl = `http://${window.location.host}/cache`;
    return { apiUrl, wsUrl, cacheUrl };
  } else {
    const apiUrl = `http://localhost:3000/api`;
    const wsUrl = `ws://localhost:3000/api`;
    const cacheUrl = `http://localhost:3000/cache`;
    return { apiUrl, wsUrl, cacheUrl };
  }
}

export const mock = initMock();
