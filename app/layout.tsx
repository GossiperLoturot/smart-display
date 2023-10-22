import type { Metadata } from "next";
import "./globals.css";

export const metadata: Metadata = { title: "smart-display" };

type RootLayoutProps = { children: React.ReactNode };

export default function RootLayout(props: RootLayoutProps) {
  return (
    <html lang="en">
      <body>{props.children}</body>
    </html>
  );
}
