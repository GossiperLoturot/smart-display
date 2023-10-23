export const dynamic = "force-dynamic";

import { NextResponse } from "next/server";
import { poll, useAppState } from "./context";

export async function GET(): Promise<NextResponse> {
  const appState = await useAppState();

  const polling = poll(appState);

  return NextResponse.json(polling);
}
