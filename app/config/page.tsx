import Link from "next/link";
import { ConfigForm } from "./component";

export default function Home() {
  return (
    <main>
      <ConfigForm />
      <Link href="/">top</Link>
    </main>
  );
}
