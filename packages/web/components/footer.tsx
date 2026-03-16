import { SITE } from "@/lib/constants";

export function Footer() {
  return (
    <footer className="border-t border-slate-200 py-8 px-4 sm:px-6 lg:px-8">
      <div className="max-w-6xl mx-auto flex flex-col sm:flex-row items-center justify-between gap-4">
        {/* Left */}
        <p className="text-sm text-slate-500">
          &copy; 2026 Aletheya. Built by YGN.
        </p>

        {/* Right */}
        <div className="flex items-center gap-6">
          <a
            href={SITE.github}
            target="_blank"
            rel="noopener noreferrer"
            className="text-sm text-slate-500 hover:text-slate-900 transition-colors"
          >
            GitHub
          </a>
          <a
            href={SITE.contact}
            className="text-sm text-slate-500 hover:text-slate-900 transition-colors"
          >
            Contact
          </a>
        </div>
      </div>
    </footer>
  );
}
