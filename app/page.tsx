import { ClockBlock, BackgroundBlock } from "./component";
import { ReactNode } from "react";

export default function Home(): ReactNode {
  return (
    <main>
      <ClockBlock />
      <BackgroundBlock />
    </main>
  );
}
