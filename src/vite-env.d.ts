/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_HTTP_API: string;
  readonly VITE_WS_API: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
