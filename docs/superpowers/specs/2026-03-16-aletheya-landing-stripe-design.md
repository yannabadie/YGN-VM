# Aletheya Landing Page + Stripe — Design Specification

**Date**: 2026-03-16
**Status**: Approved
**Author**: Yann Abadie + Claude Opus 4.6

## 1. Overview

A minimal landing page for **Aletheya** (aletheya.dev) — the verifiable proof layer for coding agents. The page presents the product, shows how it works, and offers two paid plans via Stripe Payment Links with PayPal support.

**What it is**: A static marketing site with payment links.
**What it is NOT**: A SaaS platform, a dashboard, an app with auth.

## 2. Architecture

```
aletheya.dev (domain)
    │
    ▼
GCP Cloud Run (serves static files)
    │
    ▼
Next.js static export (output: 'export')
    ├── /              → Landing page
    ├── /success       → Post-payment thank you
    └── /cancel        → Payment cancelled redirect

"Buy" buttons → Stripe Payment Links (hosted by Stripe)
    ├── Sprint: one-time €2,500 (card + PayPal)
    └── Self-Hosted: subscription €500/month (card + PayPal)
```

### Key decisions

- **No API routes.** Static export only. Zero server-side code.
- **No Stripe secret key.** Payment Links are configured in Stripe Dashboard.
- **No backend.** Cloud Run serves static files via `serve`.
- **No auth.** No user accounts, no login.

## 3. Tech Stack

- **Framework**: Next.js 16 (App Router, `output: 'export'`)
- **Styling**: Tailwind CSS
- **Hosting**: GCP Cloud Run (project: `aletheya-dev`, region: `europe-west1`)
- **Container**: Docker multi-stage (node:20-alpine → serve)
- **Payment**: Stripe Payment Links (created in Dashboard)
- **Domain**: aletheya.dev

## 4. Project Structure

```
packages/
  web/
    next.config.js              # output: 'export'
    tailwind.config.js
    postcss.config.js
    package.json
    Dockerfile
    app/
      layout.tsx                # Root layout, fonts, metadata, OG tags
      page.tsx                  # Landing page (all sections)
      success/
        page.tsx                # Post-payment confirmation
      cancel/
        page.tsx                # Payment cancelled
    components/
      navbar.tsx                # Sticky nav: logo, links, CTA
      hero.tsx                  # Hero with tagline + terminal mock
      problem.tsx               # 3-column problem statement
      how-it-works.tsx          # 3-step capture → seal → verify
      evidence-demo.tsx         # Visual evidence pack mock
      pricing.tsx               # 2 pricing cards with Payment Link buttons
      footer.tsx                # Footer with links
    public/
      favicon.ico
      og-image.png
```

## 5. Landing Page Sections

### 5.1 Navbar (sticky)

- **Left**: "Aletheya" logo (text, no image)
- **Center/Right**: How it works · Pricing · GitHub (external link to repo)
- **Far right**: "Get Started" button → scrolls to pricing
- Sticky on scroll, subtle border-bottom on scroll

### 5.2 Hero

- **Title**: "Don't just log agent actions. Prove them."
- **Subtitle**: "Cryptographic evidence packs for AI coding workflows. Signed receipts, hash chains, and tamper-evident audit artifacts for Claude Code, Copilot, and Cursor."
- **CTAs**: "View Pricing" (scroll to pricing) + "View on GitHub" (external)
- **Visual**: Terminal mock showing `aletheia verify` output with green checkmarks:
  ```
  $ aletheia verify pr-42.aletheia.json
  ✓ Chain integrity: 23/23 receipts valid
  ✓ Merkle root: verified
  ✓ Ed25519 signatures: 1/1 valid

  Evidence pack verified.
  ```

### 5.3 Problem (3 columns)

| Column | Title | Text |
|--------|-------|------|
| 1 | Logs aren't proof | Agent logs can be altered, truncated, or lost. Audit teams need tamper-evident artifacts, not raw text files. |
| 2 | Compliance demands evidence | SOC 2, ISO 27001, and the EU AI Act require traceable, verifiable records of automated actions. |
| 3 | Agents act fast, humans audit slow | AI coding agents make hundreds of decisions per session. Bridge the gap with cryptographic receipts. |

### 5.4 How it works (3 steps)

1. **Capture** — Pipe agent output into Aletheya. Each event gets a SHA-256 hash and chain link.
   - Code: `git log --format=json | aletheia capture --session pr-42`
2. **Seal** — Compute Merkle root and sign with Ed25519. Produces a portable evidence pack.
   - Code: `aletheia seal --session pr-42 --key signing.sec`
3. **Verify** — Anyone can verify the pack independently. Tamper = instant detection.
   - Code: `aletheia verify pr-42.aletheia.json`

### 5.5 Evidence Pack Demo

Visual mock of the HTML evidence report. Shows:
- Header with session name and "VERIFIED" badge
- Summary table (events, chain status, merkle root, signatures)
- Timeline of events
- Caption: "Human-readable HTML reports. Machine-verifiable JSON. Export both."

### 5.6 Pricing (2 cards)

#### Card 1 — Agent Evidence Sprint
- **Price**: €2,500 (one-time)
- **Description**: "We instrument proof on your workflows"
- **Includes**:
  - Audit of 1-2 agent workflows
  - Evidence pack generation on real flows
  - Export demo + documentation
  - Policy and integration recommendations
- **CTA**: "Book a Sprint" → Stripe Payment Link (one-time, card + PayPal)

#### Card 2 — Self-Hosted
- **Price**: €500/month
- **Description**: "Run Aletheya on your infrastructure"
- **Includes**:
  - CLI + verification tooling
  - Evidence pack export (JSON + HTML + Markdown)
  - Installation support
  - Updates and limited assistance
- **CTA**: "Subscribe" → Stripe Payment Link (subscription, card + PayPal)

Both cards: subtle highlight on the recommended plan (Sprint for quick wins).

### 5.7 Footer

- Left: "© 2026 Aletheya" + "Built by YGN"
- Center: GitHub · Contact (mailto:yann.abadie@gmail.com)
- Right: minimal legal (Privacy Policy placeholder link)

## 6. Secondary Pages

### /success

- **Title**: "Thank you!"
- **Sprint**: "We'll reach out within 24 hours to schedule your Evidence Sprint."
- **Self-Hosted**: "Check your email for setup instructions."
- Differentiates via `?plan=sprint` or `?plan=self-hosted` query param
- Link back to homepage

### /cancel

- Redirects to `/` with no visible error. Soft re-engagement.
- Or simple page: "Payment cancelled. No charges were made." + link to homepage.

## 7. Stripe Configuration

### Products (created in Stripe Dashboard)

**Product 1**: "Agent Evidence Sprint"
- Price: €2,500 EUR, one-time
- Payment Link: card + PayPal enabled
- After payment redirect: `https://aletheya.dev/success?plan=sprint`

**Product 2**: "Aletheya Self-Hosted"
- Price: €500 EUR/month, recurring
- Payment Link: card + PayPal enabled
- After payment redirect: `https://aletheya.dev/success?plan=self-hosted`

### PayPal Settlement

Configure in Stripe Dashboard > Settings > Payment methods > PayPal:
- Settlement preference: **settle on PayPal** (funds stay in PayPal balance)

### No webhooks in v1

Post-payment follow-up is manual (email notification from Stripe). Webhooks can be added later.

## 8. Visual Style

**Direction**: Clean Enterprise (Stripe/Datadog-like)

- **Background**: White (#ffffff) with subtle gray sections (#f8fafc)
- **Text**: Dark slate (#0f172a) for headings, medium (#475569) for body
- **Accent**: Blue (#2563eb) for CTAs and links
- **Success green**: #16a34a for verification checkmarks
- **Font**: Inter (Google Fonts) — clean, professional, widely used in B2B SaaS
- **Cards**: White with subtle border (#e2e8f0) and soft shadow
- **Terminal mock**: Dark (#0f172a) with monospace font, colored output
- **Responsive**: Mobile-first, breakpoints at sm/md/lg
- **Dark mode**: Not in v1 (light only, simpler)

## 9. Docker & Deployment

### Dockerfile

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

### Cloud Run deployment

```bash
# Build and push image
gcloud builds submit --tag gcr.io/aletheya-dev/aletheya-web

# Deploy
gcloud run deploy aletheya-web \
  --image gcr.io/aletheya-dev/aletheya-web \
  --platform managed \
  --region europe-west1 \
  --allow-unauthenticated \
  --port 8080

# Map custom domain
gcloud run domain-mappings create \
  --service aletheya-web \
  --domain aletheya.dev \
  --region europe-west1
```

### GCP project

- Project ID: `aletheya-dev`
- Region: `europe-west1`
- Services needed: Cloud Run, Artifact Registry (or Container Registry)
- Auth workaround: service account key (SSL proxy issue blocks CLI auth)

## 10. SEO & Metadata

```html
<title>Aletheya — Verifiable Proof for AI Coding Agents</title>
<meta name="description" content="Cryptographic evidence packs for coding-agent compliance. Signed receipts, hash chains, and tamper-evident audit artifacts." />
<meta property="og:title" content="Aletheya — Don't just log agent actions. Prove them." />
<meta property="og:description" content="Cryptographic evidence packs for AI coding workflows." />
<meta property="og:image" content="https://aletheya.dev/og-image.png" />
<meta property="og:url" content="https://aletheya.dev" />
```

## 11. Out of Scope (v1)

- Dark mode
- Blog / docs section
- User accounts / auth
- Stripe webhooks
- Analytics (add later: Plausible or simple GA)
- i18n / French version
- Contact form (email link suffices)
- Animated demos / video

## 12. Success Criteria

- Landing page loads in < 2 seconds
- Both Payment Links work (one-time + subscription)
- PayPal option visible in Stripe Checkout
- Pages render correctly on mobile
- /success and /cancel pages work with query params
- Deployed on Cloud Run with aletheya.dev domain
