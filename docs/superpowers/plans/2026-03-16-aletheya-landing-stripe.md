# Aletheya Landing Page + Stripe Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a static Next.js landing page for aletheya.dev with Stripe Payment Links (card + PayPal) for two product tiers, deployed to GCP Cloud Run.

**Architecture:** Next.js App Router with `output: 'export'` produces static HTML/CSS/JS. Stripe Payment Links (configured in Dashboard, no backend) handle payments. Docker container with `serve` serves the static files on Cloud Run.

**Tech Stack:** Next.js 16, React 19, Tailwind CSS, TypeScript, Docker, GCP Cloud Run, Stripe Payment Links

**Spec:** `docs/superpowers/specs/2026-03-16-aletheya-landing-stripe-design.md`

---

## File Structure

```
packages/
  web/
    package.json                    # Next.js + Tailwind deps
    next.config.js                  # output: 'export'
    tailwind.config.ts              # Tailwind config with custom colors
    postcss.config.js               # PostCSS for Tailwind
    tsconfig.json                   # TypeScript config
    Dockerfile                      # Multi-stage: build → serve
    app/
      layout.tsx                    # Root layout: Inter font, metadata, OG
      page.tsx                      # Landing page: assembles all sections
      success/
        page.tsx                    # Post-payment thank you
      cancel/
        page.tsx                    # Payment cancelled
      globals.css                   # Tailwind directives + base styles
    components/
      navbar.tsx                    # Sticky nav with logo, links, CTA
      hero.tsx                      # Hero: tagline + terminal mock
      problem.tsx                   # 3-column problem statement
      how-it-works.tsx              # 3-step flow with code snippets
      evidence-demo.tsx             # Evidence pack visual mock
      pricing.tsx                   # 2 pricing cards with Payment Link buttons
      footer.tsx                    # Footer with links
    lib/
      constants.ts                  # Payment links, URLs, copy strings
    public/
      favicon.ico                   # Favicon
```

---

## Chunk 1: Project Setup + Layout

### Task 1: Initialize Next.js Project

**Files:**
- Create: `packages/web/package.json`
- Create: `packages/web/next.config.js`
- Create: `packages/web/tailwind.config.ts`
- Create: `packages/web/postcss.config.js`
- Create: `packages/web/tsconfig.json`

- [ ] **Step 1: Create package.json**

```json
{
  "name": "aletheya-web",
  "version": "0.1.0",
  "private": true,
  "scripts": {
    "dev": "next dev",
    "build": "next build",
    "start": "next start"
  },
  "dependencies": {
    "next": "^16.0.0",
    "react": "^19.0.0",
    "react-dom": "^19.0.0"
  },
  "devDependencies": {
    "@types/node": "^22.0.0",
    "@types/react": "^19.0.0",
    "@types/react-dom": "^19.0.0",
    "typescript": "^5.7.0",
    "tailwindcss": "^4.0.0",
    "@tailwindcss/postcss": "^4.0.0"
  }
}
```

- [ ] **Step 2: Create next.config.js**

```javascript
/** @type {import('next').NextConfig} */
const nextConfig = {
  output: 'export',
  images: {
    unoptimized: true,
  },
};

module.exports = nextConfig;
```

- [ ] **Step 3: Create postcss.config.js**

```javascript
module.exports = {
  plugins: {
    "@tailwindcss/postcss": {},
  },
};
```

- [ ] **Step 4: Create tailwind.config.ts**

```typescript
import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          50: "#eff6ff",
          500: "#2563eb",
          600: "#1d4ed8",
          700: "#1e40af",
        },
        slate: {
          50: "#f8fafc",
          100: "#f1f5f9",
          200: "#e2e8f0",
          400: "#94a3b8",
          500: "#64748b",
          700: "#334155",
          800: "#1e293b",
          900: "#0f172a",
        },
        success: "#16a34a",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["Fira Code", "Cascadia Code", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
```

- [ ] **Step 5: Create tsconfig.json**

```json
{
  "compilerOptions": {
    "target": "ES2017",
    "lib": ["dom", "dom.iterable", "esnext"],
    "allowJs": true,
    "skipLibCheck": true,
    "strict": true,
    "noEmit": true,
    "esModuleInterop": true,
    "module": "esnext",
    "moduleResolution": "bundler",
    "resolveJsonModule": true,
    "isolatedModules": true,
    "jsx": "preserve",
    "incremental": true,
    "paths": {
      "@/*": ["./*"]
    },
    "plugins": [{ "name": "next" }]
  },
  "include": ["next-env.d.ts", "**/*.ts", "**/*.tsx"],
  "exclude": ["node_modules"]
}
```

- [ ] **Step 6: Install dependencies**

Run: `cd packages/web && npm install`
Expected: `node_modules` created, no errors.

- [ ] **Step 7: Commit**

```bash
git add packages/web/package.json packages/web/package-lock.json packages/web/next.config.js packages/web/tailwind.config.ts packages/web/postcss.config.js packages/web/tsconfig.json
git commit -m "feat(web): initialize Next.js project with Tailwind CSS"
```

---

### Task 2: Root Layout + Globals + Constants

**Files:**
- Create: `packages/web/app/globals.css`
- Create: `packages/web/app/layout.tsx`
- Create: `packages/web/lib/constants.ts`

- [ ] **Step 1: Create globals.css**

```css
@import "tailwindcss";

html {
  scroll-behavior: smooth;
}

body {
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}
```

- [ ] **Step 2: Create constants.ts**

```typescript
export const SITE = {
  name: "Aletheya",
  tagline: "Don't just log agent actions. Prove them.",
  description:
    "Cryptographic evidence packs for AI coding workflows. Signed receipts, hash chains, and tamper-evident audit artifacts.",
  url: "https://aletheya.dev",
  github: "https://github.com/yannabadie/YGN-VM",
  contact: "mailto:yann.abadie@gmail.com",
};

// Replace these with actual Stripe Payment Link URLs after creating them in Dashboard
export const STRIPE = {
  sprintLink: "https://buy.stripe.com/REPLACE_WITH_SPRINT_LINK",
  selfHostedLink: "https://buy.stripe.com/REPLACE_WITH_SELF_HOSTED_LINK",
};

export const PRICING = {
  sprint: {
    name: "Agent Evidence Sprint",
    price: "€2,500",
    period: "one-time",
    description: "We instrument proof on your workflows",
    features: [
      "Audit of 1–2 agent workflows",
      "Evidence pack generation on real flows",
      "Export demo + documentation",
      "Policy and integration recommendations",
    ],
    cta: "Book a Sprint",
    link: "https://buy.stripe.com/REPLACE_WITH_SPRINT_LINK",
    highlighted: true,
  },
  selfHosted: {
    name: "Self-Hosted",
    price: "€500",
    period: "/month",
    description: "Run Aletheya on your infrastructure",
    features: [
      "CLI + verification tooling",
      "Evidence pack export (JSON + HTML + Markdown)",
      "Installation support",
      "Updates and limited assistance",
    ],
    cta: "Subscribe",
    link: "https://buy.stripe.com/REPLACE_WITH_SELF_HOSTED_LINK",
    highlighted: false,
  },
};
```

- [ ] **Step 3: Create layout.tsx**

```tsx
import type { Metadata } from "next";
import { Inter } from "next/font/google";
import "./globals.css";
import { SITE } from "@/lib/constants";

const inter = Inter({ subsets: ["latin"] });

export const metadata: Metadata = {
  title: `${SITE.name} — Verifiable Proof for AI Coding Agents`,
  description: SITE.description,
  openGraph: {
    title: `${SITE.name} — ${SITE.tagline}`,
    description: SITE.description,
    url: SITE.url,
    siteName: SITE.name,
    images: [{ url: `${SITE.url}/og-image.png`, width: 1200, height: 630 }],
    type: "website",
  },
  twitter: {
    card: "summary_large_image",
    title: `${SITE.name} — ${SITE.tagline}`,
    description: SITE.description,
  },
};

export default function RootLayout({
  children,
}: {
  children: React.ReactNode;
}) {
  return (
    <html lang="en" className={inter.className}>
      <body className="bg-white text-slate-900 antialiased">{children}</body>
    </html>
  );
}
```

- [ ] **Step 4: Create minimal page.tsx to test**

```tsx
export default function Home() {
  return (
    <main>
      <h1 className="text-4xl font-bold p-8">Aletheya</h1>
    </main>
  );
}
```

- [ ] **Step 5: Verify dev server works**

Run: `cd packages/web && npm run dev`
Expected: Next.js dev server starts, page loads at http://localhost:3000 with "Aletheya" heading.

- [ ] **Step 6: Verify static export**

Run: `cd packages/web && npm run build`
Expected: `out/` directory created with `index.html`.

- [ ] **Step 7: Commit**

```bash
git add packages/web/app/ packages/web/lib/
git commit -m "feat(web): add root layout, globals, constants, and minimal page"
```

---

## Chunk 2: Landing Page Components

### Task 3: Navbar

**Files:**
- Create: `packages/web/components/navbar.tsx`

- [ ] **Step 1: Write navbar component**

```tsx
"use client";

import { useState, useEffect } from "react";
import { SITE } from "@/lib/constants";

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 10);
    window.addEventListener("scroll", onScroll);
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <nav
      className={`fixed top-0 left-0 right-0 z-50 transition-all ${
        scrolled
          ? "bg-white/95 backdrop-blur-sm border-b border-slate-200"
          : "bg-transparent"
      }`}
    >
      <div className="max-w-6xl mx-auto px-6 h-16 flex items-center justify-between">
        <a href="/" className="text-xl font-bold text-slate-900">
          {SITE.name}
        </a>

        <div className="hidden md:flex items-center gap-8">
          <a
            href="#how-it-works"
            className="text-sm text-slate-600 hover:text-slate-900 transition"
          >
            How it works
          </a>
          <a
            href="#pricing"
            className="text-sm text-slate-600 hover:text-slate-900 transition"
          >
            Pricing
          </a>
          <a
            href={SITE.github}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-slate-600 hover:text-slate-900 transition"
          >
            GitHub
          </a>
          <a
            href="#pricing"
            className="inline-flex items-center px-4 py-2 text-sm font-medium text-white bg-brand-500 rounded-lg hover:bg-brand-600 transition"
          >
            Get Started
          </a>
        </div>
      </div>
    </nav>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/navbar.tsx
git commit -m "feat(web): add sticky navbar component"
```

---

### Task 4: Hero Section

**Files:**
- Create: `packages/web/components/hero.tsx`

- [ ] **Step 1: Write hero component**

```tsx
import { SITE } from "@/lib/constants";

export function Hero() {
  return (
    <section className="pt-32 pb-20 px-6">
      <div className="max-w-6xl mx-auto grid lg:grid-cols-2 gap-12 items-center">
        <div>
          <h1 className="text-4xl md:text-5xl font-bold text-slate-900 leading-tight">
            Don&apos;t just log agent actions.{" "}
            <span className="text-brand-500">Prove them.</span>
          </h1>
          <p className="mt-6 text-lg text-slate-500 max-w-xl">
            Cryptographic evidence packs for AI coding workflows. Signed
            receipts, hash chains, and tamper-evident audit artifacts for Claude
            Code, Copilot, and Cursor.
          </p>
          <div className="mt-8 flex flex-wrap gap-4">
            <a
              href="#pricing"
              className="inline-flex items-center px-6 py-3 text-sm font-medium text-white bg-brand-500 rounded-lg hover:bg-brand-600 transition shadow-sm"
            >
              View Pricing
            </a>
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center px-6 py-3 text-sm font-medium text-slate-700 bg-white border border-slate-200 rounded-lg hover:bg-slate-50 transition"
            >
              View on GitHub
            </a>
          </div>
        </div>

        <div className="bg-slate-900 rounded-xl p-6 font-mono text-sm shadow-2xl">
          <div className="flex items-center gap-2 mb-4">
            <div className="w-3 h-3 rounded-full bg-red-500" />
            <div className="w-3 h-3 rounded-full bg-yellow-500" />
            <div className="w-3 h-3 rounded-full bg-green-500" />
            <span className="ml-2 text-slate-400 text-xs">terminal</span>
          </div>
          <div className="space-y-1">
            <p className="text-slate-400">
              $ aletheia verify pr-42.aletheia.json
            </p>
            <p className="text-green-400">
              ✓ Chain integrity: 23/23 receipts valid
            </p>
            <p className="text-green-400">✓ Merkle root: verified</p>
            <p className="text-green-400">
              ✓ Ed25519 signatures: 1/1 valid
            </p>
            <p className="mt-3 text-white font-semibold">
              Evidence pack verified.
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/hero.tsx
git commit -m "feat(web): add hero section with terminal mock"
```

---

### Task 5: Problem Section

**Files:**
- Create: `packages/web/components/problem.tsx`

- [ ] **Step 1: Write problem component**

```tsx
const PROBLEMS = [
  {
    title: "Logs aren't proof",
    text: "Agent logs can be altered, truncated, or lost. Audit teams need tamper-evident artifacts, not raw text files.",
    icon: "📄",
  },
  {
    title: "Compliance demands evidence",
    text: "SOC 2, ISO 27001, and the EU AI Act require traceable, verifiable records of automated actions.",
    icon: "🔒",
  },
  {
    title: "Agents act fast, humans audit slow",
    text: "AI coding agents make hundreds of decisions per session. Bridge the gap with cryptographic receipts.",
    icon: "⚡",
  },
];

export function Problem() {
  return (
    <section className="py-20 px-6 bg-slate-50">
      <div className="max-w-6xl mx-auto">
        <h2 className="text-3xl font-bold text-center text-slate-900">
          Why evidence packs?
        </h2>
        <p className="mt-4 text-center text-slate-500 max-w-2xl mx-auto">
          Monitoring tells you what happened. Aletheya proves it.
        </p>
        <div className="mt-12 grid md:grid-cols-3 gap-8">
          {PROBLEMS.map((p) => (
            <div
              key={p.title}
              className="bg-white rounded-xl p-6 border border-slate-200 shadow-sm"
            >
              <div className="text-2xl mb-3">{p.icon}</div>
              <h3 className="text-lg font-semibold text-slate-900">
                {p.title}
              </h3>
              <p className="mt-2 text-sm text-slate-500">{p.text}</p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/problem.tsx
git commit -m "feat(web): add problem section"
```

---

### Task 6: How It Works

**Files:**
- Create: `packages/web/components/how-it-works.tsx`

- [ ] **Step 1: Write how-it-works component**

```tsx
const STEPS = [
  {
    step: "1",
    title: "Capture",
    description:
      "Pipe agent output into Aletheya. Each event gets a SHA-256 hash and chain link.",
    code: "git log --format=json | aletheia capture --session pr-42",
  },
  {
    step: "2",
    title: "Seal",
    description:
      "Compute Merkle root and sign with Ed25519. Produces a portable evidence pack.",
    code: "aletheia seal --session pr-42 --key signing.sec",
  },
  {
    step: "3",
    title: "Verify",
    description:
      "Anyone can verify the pack independently. Tamper = instant detection.",
    code: "aletheia verify pr-42.aletheia.json",
  },
];

export function HowItWorks() {
  return (
    <section id="how-it-works" className="py-20 px-6">
      <div className="max-w-6xl mx-auto">
        <h2 className="text-3xl font-bold text-center text-slate-900">
          How it works
        </h2>
        <p className="mt-4 text-center text-slate-500 max-w-2xl mx-auto">
          Three commands. Tamper-evident proof.
        </p>
        <div className="mt-12 grid md:grid-cols-3 gap-8">
          {STEPS.map((s) => (
            <div key={s.step} className="relative">
              <div className="inline-flex items-center justify-center w-10 h-10 rounded-full bg-brand-500 text-white font-bold text-sm mb-4">
                {s.step}
              </div>
              <h3 className="text-lg font-semibold text-slate-900">
                {s.title}
              </h3>
              <p className="mt-2 text-sm text-slate-500">{s.description}</p>
              <div className="mt-4 bg-slate-900 rounded-lg p-3 font-mono text-xs text-green-400 overflow-x-auto">
                $ {s.code}
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/how-it-works.tsx
git commit -m "feat(web): add how-it-works section with code snippets"
```

---

### Task 7: Evidence Demo

**Files:**
- Create: `packages/web/components/evidence-demo.tsx`

- [ ] **Step 1: Write evidence-demo component**

```tsx
export function EvidenceDemo() {
  return (
    <section className="py-20 px-6 bg-slate-50">
      <div className="max-w-6xl mx-auto">
        <h2 className="text-3xl font-bold text-center text-slate-900">
          Human-readable. Machine-verifiable.
        </h2>
        <p className="mt-4 text-center text-slate-500 max-w-2xl mx-auto">
          Every evidence pack exports as JSON, Markdown, and a standalone HTML
          report.
        </p>

        <div className="mt-12 max-w-3xl mx-auto bg-white rounded-xl border border-slate-200 shadow-lg overflow-hidden">
          {/* Mock report header */}
          <div className="px-6 py-4 border-b border-slate-200 flex items-center justify-between">
            <div>
              <h3 className="font-semibold text-slate-900">
                Evidence Report — pr-42
              </h3>
              <p className="text-xs text-slate-400">
                2026-03-16 14:32 UTC
              </p>
            </div>
            <span className="inline-flex items-center px-3 py-1 rounded-full text-xs font-medium bg-green-100 text-green-800">
              VERIFIED
            </span>
          </div>

          {/* Mock summary */}
          <div className="px-6 py-4 grid grid-cols-2 md:grid-cols-4 gap-4 border-b border-slate-100">
            <div>
              <p className="text-xs text-slate-400 uppercase">Events</p>
              <p className="text-lg font-semibold text-slate-900">23</p>
            </div>
            <div>
              <p className="text-xs text-slate-400 uppercase">Chain</p>
              <p className="text-lg font-semibold text-green-600">
                23/23 valid
              </p>
            </div>
            <div>
              <p className="text-xs text-slate-400 uppercase">Merkle Root</p>
              <p className="text-sm font-mono text-brand-500 truncate">
                a3f2b8c4...
              </p>
            </div>
            <div>
              <p className="text-xs text-slate-400 uppercase">Signatures</p>
              <p className="text-lg font-semibold text-green-600">1/1</p>
            </div>
          </div>

          {/* Mock timeline */}
          <div className="px-6 py-4">
            <p className="text-xs text-slate-400 uppercase mb-3">
              Timeline (excerpt)
            </p>
            <div className="space-y-2 text-xs font-mono">
              <div className="flex gap-3">
                <span className="text-slate-400">#0</span>
                <span className="text-brand-500">shell_exec</span>
                <span className="text-slate-600 truncate">
                  cargo test --all
                </span>
              </div>
              <div className="flex gap-3">
                <span className="text-slate-400">#1</span>
                <span className="text-brand-500">file_edit</span>
                <span className="text-slate-600 truncate">
                  src/auth.rs (+42 -3)
                </span>
              </div>
              <div className="flex gap-3">
                <span className="text-slate-400">#2</span>
                <span className="text-brand-500">test_run</span>
                <span className="text-green-600 truncate">
                  All 156 tests passed
                </span>
              </div>
            </div>
          </div>
        </div>

        <div className="mt-6 text-center">
          <p className="text-sm text-slate-500">
            SHA-256 hash chain · Merkle root · Ed25519 signatures · Standalone
            HTML export
          </p>
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/evidence-demo.tsx
git commit -m "feat(web): add evidence pack demo section"
```

---

### Task 8: Pricing Section

**Files:**
- Create: `packages/web/components/pricing.tsx`

- [ ] **Step 1: Write pricing component**

```tsx
import { PRICING } from "@/lib/constants";

const plans = [PRICING.sprint, PRICING.selfHosted];

export function Pricing() {
  return (
    <section id="pricing" className="py-20 px-6">
      <div className="max-w-6xl mx-auto">
        <h2 className="text-3xl font-bold text-center text-slate-900">
          Pricing
        </h2>
        <p className="mt-4 text-center text-slate-500 max-w-2xl mx-auto">
          Start with a guided sprint, or run Aletheya on your own
          infrastructure.
        </p>

        <div className="mt-12 grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
          {plans.map((plan) => (
            <div
              key={plan.name}
              className={`rounded-xl p-8 border ${
                plan.highlighted
                  ? "border-brand-500 shadow-lg ring-1 ring-brand-500"
                  : "border-slate-200 shadow-sm"
              }`}
            >
              {plan.highlighted && (
                <span className="inline-block mb-4 px-3 py-1 text-xs font-medium bg-brand-50 text-brand-600 rounded-full">
                  Recommended
                </span>
              )}
              <h3 className="text-xl font-bold text-slate-900">{plan.name}</h3>
              <p className="mt-1 text-sm text-slate-500">{plan.description}</p>
              <div className="mt-4">
                <span className="text-4xl font-bold text-slate-900">
                  {plan.price}
                </span>
                <span className="text-sm text-slate-400 ml-1">
                  {plan.period}
                </span>
              </div>
              <ul className="mt-6 space-y-3">
                {plan.features.map((f) => (
                  <li
                    key={f}
                    className="flex items-start gap-2 text-sm text-slate-600"
                  >
                    <span className="text-green-500 mt-0.5">✓</span>
                    {f}
                  </li>
                ))}
              </ul>
              <a
                href={plan.link}
                target="_blank"
                rel="noopener noreferrer"
                className={`mt-8 block text-center px-6 py-3 text-sm font-medium rounded-lg transition ${
                  plan.highlighted
                    ? "text-white bg-brand-500 hover:bg-brand-600 shadow-sm"
                    : "text-slate-700 bg-white border border-slate-200 hover:bg-slate-50"
                }`}
              >
                {plan.cta}
              </a>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/pricing.tsx
git commit -m "feat(web): add pricing section with two plans"
```

---

### Task 9: Footer

**Files:**
- Create: `packages/web/components/footer.tsx`

- [ ] **Step 1: Write footer component**

```tsx
import { SITE } from "@/lib/constants";

export function Footer() {
  return (
    <footer className="py-12 px-6 border-t border-slate-200">
      <div className="max-w-6xl mx-auto flex flex-col md:flex-row items-center justify-between gap-4">
        <p className="text-sm text-slate-400">
          © 2026 {SITE.name}. Built by YGN.
        </p>
        <div className="flex items-center gap-6">
          <a
            href={SITE.github}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-slate-400 hover:text-slate-600 transition"
          >
            GitHub
          </a>
          <a
            href={SITE.contact}
            className="text-sm text-slate-400 hover:text-slate-600 transition"
          >
            Contact
          </a>
        </div>
      </div>
    </footer>
  );
}
```

- [ ] **Step 2: Commit**

```bash
git add packages/web/components/footer.tsx
git commit -m "feat(web): add footer component"
```

---

## Chunk 3: Page Assembly + Secondary Pages + Docker

### Task 10: Assemble Landing Page

**Files:**
- Modify: `packages/web/app/page.tsx`

- [ ] **Step 1: Assemble all sections**

```tsx
import { Navbar } from "@/components/navbar";
import { Hero } from "@/components/hero";
import { Problem } from "@/components/problem";
import { HowItWorks } from "@/components/how-it-works";
import { EvidenceDemo } from "@/components/evidence-demo";
import { Pricing } from "@/components/pricing";
import { Footer } from "@/components/footer";

export default function Home() {
  return (
    <>
      <Navbar />
      <main>
        <Hero />
        <Problem />
        <HowItWorks />
        <EvidenceDemo />
        <Pricing />
      </main>
      <Footer />
    </>
  );
}
```

- [ ] **Step 2: Verify dev server renders all sections**

Run: `cd packages/web && npm run dev`
Expected: All 7 sections visible, smooth scroll works for anchor links, responsive on mobile.

- [ ] **Step 3: Verify static export**

Run: `cd packages/web && npm run build`
Expected: `out/` directory with `index.html` containing all sections.

- [ ] **Step 4: Commit**

```bash
git add packages/web/app/page.tsx
git commit -m "feat(web): assemble landing page with all sections"
```

---

### Task 11: Success + Cancel Pages

**Files:**
- Create: `packages/web/app/success/page.tsx`
- Create: `packages/web/app/cancel/page.tsx`

- [ ] **Step 1: Write success page**

```tsx
"use client";

import { useSearchParams } from "next/navigation";
import { Suspense } from "react";
import { SITE } from "@/lib/constants";

function SuccessContent() {
  const params = useSearchParams();
  const plan = params.get("plan");

  const message =
    plan === "sprint"
      ? "We'll reach out within 24 hours to schedule your Evidence Sprint."
      : plan === "self-hosted"
        ? "Check your email for setup instructions."
        : "We'll be in touch shortly.";

  return (
    <div className="min-h-screen flex items-center justify-center px-6">
      <div className="text-center max-w-md">
        <div className="text-5xl mb-6">✓</div>
        <h1 className="text-3xl font-bold text-slate-900">Thank you!</h1>
        <p className="mt-4 text-slate-500">{message}</p>
        <a
          href="/"
          className="mt-8 inline-flex items-center px-6 py-3 text-sm font-medium text-white bg-brand-500 rounded-lg hover:bg-brand-600 transition"
        >
          Back to {SITE.name}
        </a>
      </div>
    </div>
  );
}

export default function SuccessPage() {
  return (
    <Suspense
      fallback={
        <div className="min-h-screen flex items-center justify-center">
          <p className="text-slate-400">Loading...</p>
        </div>
      }
    >
      <SuccessContent />
    </Suspense>
  );
}
```

- [ ] **Step 2: Write cancel page**

```tsx
import { SITE } from "@/lib/constants";

export default function CancelPage() {
  return (
    <div className="min-h-screen flex items-center justify-center px-6">
      <div className="text-center max-w-md">
        <h1 className="text-3xl font-bold text-slate-900">
          Payment cancelled
        </h1>
        <p className="mt-4 text-slate-500">
          No charges were made. You can try again anytime.
        </p>
        <a
          href="/"
          className="mt-8 inline-flex items-center px-6 py-3 text-sm font-medium text-white bg-brand-500 rounded-lg hover:bg-brand-600 transition"
        >
          Back to {SITE.name}
        </a>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Verify both pages render**

Run: `cd packages/web && npm run dev`
Visit: http://localhost:3000/success?plan=sprint and http://localhost:3000/cancel
Expected: Both pages render correctly.

- [ ] **Step 4: Commit**

```bash
git add packages/web/app/success/ packages/web/app/cancel/
git commit -m "feat(web): add success and cancel pages"
```

---

### Task 12: Dockerfile

**Files:**
- Create: `packages/web/Dockerfile`

- [ ] **Step 1: Write Dockerfile**

```dockerfile
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine AS runner
WORKDIR /app
RUN npm install -g serve
COPY --from=builder /app/out ./out
EXPOSE 8080
CMD ["serve", "out", "-l", "8080", "-s"]
```

- [ ] **Step 2: Test Docker build locally (optional)**

Run: `cd packages/web && docker build -t aletheya-web .`
Expected: Image builds successfully.

Run: `docker run -p 8080:8080 aletheya-web`
Expected: Site accessible at http://localhost:8080

- [ ] **Step 3: Commit**

```bash
git add packages/web/Dockerfile
git commit -m "feat(web): add Dockerfile for Cloud Run deployment"
```

---

### Task 13: Favicon + Final Polish

**Files:**
- Create: `packages/web/public/favicon.ico` (or `favicon.svg`)
- Modify: `packages/web/app/layout.tsx` (add favicon link if needed)

- [ ] **Step 1: Create a simple SVG favicon**

Create `packages/web/app/icon.svg`:

```svg
<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 32 32">
  <rect width="32" height="32" rx="6" fill="#2563eb"/>
  <text x="50%" y="50%" text-anchor="middle" dy=".35em" font-family="system-ui" font-weight="bold" font-size="18" fill="white">A</text>
</svg>
```

- [ ] **Step 2: Final build verification**

Run: `cd packages/web && npm run build`
Expected: Clean build, `out/` directory contains index.html, success.html, cancel.html (or success/index.html etc.)

- [ ] **Step 3: Commit and push**

```bash
git add packages/web/
git commit -m "feat(web): add favicon and finalize landing page"
git push origin master
```

---

## Task Summary

| Task | Component | Steps |
|------|-----------|-------|
| 1 | Project setup (Next.js + Tailwind) | 7 |
| 2 | Layout + globals + constants | 7 |
| 3 | Navbar | 2 |
| 4 | Hero section | 2 |
| 5 | Problem section | 2 |
| 6 | How it works | 2 |
| 7 | Evidence demo | 2 |
| 8 | Pricing section | 2 |
| 9 | Footer | 2 |
| 10 | Page assembly | 4 |
| 11 | Success + cancel pages | 4 |
| 12 | Dockerfile | 3 |
| 13 | Favicon + final polish | 3 |
| **Total** | | **42 steps** |

## Post-Implementation

After the landing page is built and pushed:

1. **Buy domain** aletheya.dev (Google Domains, Namecheap, or Cloudflare Registrar)
2. **Create Stripe products** in Dashboard: Sprint (€2,500 one-time) + Self-Hosted (€500/month)
3. **Enable PayPal** in Stripe Dashboard > Payment methods
4. **Create Payment Links** for both products, set success/cancel URLs
5. **Update constants.ts** with real Payment Link URLs
6. **Deploy to Cloud Run** (resolve GCP auth first — service account key or direct web deploy)
7. **Configure DNS** for aletheya.dev → Cloud Run
