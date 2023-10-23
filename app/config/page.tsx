import { ReactNode } from "react";
import ConfigForm from "./component";

export default function Home(): ReactNode {
  return (
    <main>
      <p>Configuration</p>
      <ConfigForm />
    </main>
  );
}
