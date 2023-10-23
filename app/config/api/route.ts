import { readFile, writeFile } from "fs/promises";
import { NextRequest, NextResponse } from "next/server";
import { Config, ConfigScheme } from "../scheme";

export async function GET(): Promise<NextResponse> {
  const buf = await readFile("smart-display.dyn.json");
  const json = JSON.parse(buf.toString());
  const config = ConfigScheme.parse(json) as Config;
  return NextResponse.json(config);
}

export async function POST(req: NextRequest): Promise<NextResponse> {
  const json = await req.json();
  const config = ConfigScheme.parse(json) as Config;
  const buf = Buffer.from(JSON.stringify(config));
  await writeFile("smart-display.dyn.json", buf);
  return new NextResponse();
}
