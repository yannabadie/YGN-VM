export const SITE = {
  name: "Aletheya",
  tagline: "Don't just log agent actions. Prove them.",
  description:
    "Cryptographic evidence packs for AI coding workflows. Signed receipts, hash chains, and tamper-evident audit artifacts.",
  url: "https://aletheya.dev",
  github: "https://github.com/yannabadie/YGN-VM",
  contact: "mailto:yann.abadie@gmail.com",
};

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
