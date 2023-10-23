import { Clock, Background } from "./component";
import { ReactNode } from "react";

export default function Home(): ReactNode {
  return (
    <main>
      <Clock />
      <Background />
    </main>
  );
}
