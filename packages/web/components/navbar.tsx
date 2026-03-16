"use client";

import { useEffect, useState } from "react";
import { SITE } from "@/lib/constants";

export function Navbar() {
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const handleScroll = () => {
      setScrolled(window.scrollY > 10);
    };
    window.addEventListener("scroll", handleScroll, { passive: true });
    return () => window.removeEventListener("scroll", handleScroll);
  }, []);

  return (
    <header
      className={[
        "fixed top-0 left-0 right-0 z-50 transition-all duration-200",
        scrolled
          ? "bg-white/95 backdrop-blur-sm border-b border-slate-200 shadow-sm"
          : "bg-transparent",
      ].join(" ")}
    >
      <nav className="max-w-6xl mx-auto px-4 sm:px-6 lg:px-8 h-16 flex items-center justify-between">
        {/* Logo */}
        <a
          href="/"
          className="text-xl font-bold text-slate-900 hover:text-brand-600 transition-colors"
        >
          Aletheya
        </a>

        {/* Center-right nav links — hidden on mobile */}
        <div className="hidden md:flex items-center gap-8">
          <a
            href="#how-it-works"
            className="text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors"
          >
            How it works
          </a>
          <a
            href="#pricing"
            className="text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors"
          >
            Pricing
          </a>
          <a
            href={SITE.github}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors"
          >
            GitHub
          </a>
        </div>

        {/* CTA button */}
        <a
          href="#pricing"
          className="inline-flex items-center px-4 py-2 rounded-lg bg-brand-500 text-white text-sm font-semibold hover:bg-brand-600 active:bg-brand-700 transition-colors"
        >
          Get Started
        </a>
      </nav>
    </header>
  );
}
