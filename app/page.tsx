import Link from "next/link";
import { Background, Clock } from "./component";

export default function Home() {
  return (
    <main>
      <Clock />
      <Background />
      <Link href="/config">config</Link>
    </main>
  );
}
