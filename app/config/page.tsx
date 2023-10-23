import ConfigForm from "./component";
import { ReactNode } from "react";

export default function Home(): ReactNode {
  return (
    <main>
      <p>Configuration</p>
      <ConfigForm />
    </main>
  );
}
