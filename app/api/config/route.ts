import { NextRequest, NextResponse } from "next/server";
import { Config, ConfigScheme } from "../scheme";
import { useAppState, writeConfig } from "../context";

export async function GET(): Promise<NextResponse> {
  const appState = await useAppState();

  const config = appState.config;

  return NextResponse.json(config);
}

export async function POST(req: NextRequest): Promise<NextResponse> {
  const appState = await useAppState();

  const json = await req.json();
  const config = ConfigScheme.parse(json) as Config;
  appState.config = config;
  appState.imageIndex = 0;

  writeConfig(config);

  return new NextResponse();
}
