export type Mock = {
  apiUrl: string;
  wsUrl: string;
};

function initMock(): Mock {
  if (process.env.NODE_ENV === "development") {
    const apiUrl = `http://localhost:3000/api`;
    const wsUrl = `ws://localhost:3000/api`;
    return { apiUrl, wsUrl };
  }

  const apiUrl = `http://${window.location.host}/api`;
  const wsUrl = `ws://${window.location.host}/api`;
  return { apiUrl, wsUrl };
}

export const mock = initMock();
