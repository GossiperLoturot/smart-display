import type { Metadata } from "next";
import { Noto_Sans } from "next/font/google";
import { ReactNode } from "react";

const font = Noto_Sans({ weight: "400", subsets: [] });

export const metadata: Metadata = { title: "smart-display" };

export type RootLayoutProps = { children: ReactNode };

export default function RootLayout(props: RootLayoutProps): ReactNode {
  return (
    <html lang="en">
      <body className={font.className}>{props.children}</body>
    </html>
  );
}
