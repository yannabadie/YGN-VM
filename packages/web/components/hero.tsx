import { SITE } from "@/lib/constants";

export function Hero() {
  return (
    <section className="pt-32 pb-20 px-4 sm:px-6 lg:px-8 max-w-6xl mx-auto">
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-12 items-center">
        {/* Left column */}
        <div className="flex flex-col gap-6">
          <h1 className="text-4xl sm:text-5xl font-bold leading-tight text-slate-900">
            Don&apos;t just log agent actions.{" "}
            <span className="text-brand-500">Prove them.</span>
          </h1>
          <p className="text-lg text-slate-600 leading-relaxed">
            {SITE.description}
          </p>
          <div className="flex flex-col sm:flex-row gap-3 pt-2">
            <a
              href="#pricing"
              className="inline-flex items-center justify-center px-6 py-3 rounded-lg bg-brand-500 text-white font-semibold hover:bg-brand-600 active:bg-brand-700 transition-colors"
            >
              View Pricing
            </a>
            <a
              href={SITE.github}
              target="_blank"
              rel="noopener noreferrer"
              className="inline-flex items-center justify-center px-6 py-3 rounded-lg border border-slate-300 text-slate-700 font-semibold hover:border-slate-400 hover:bg-slate-50 transition-colors"
            >
              View on GitHub
            </a>
          </div>
        </div>

        {/* Right column — terminal mock */}
        <div className="rounded-xl overflow-hidden shadow-2xl border border-slate-200">
          {/* Window chrome */}
          <div className="bg-slate-800 px-4 py-3 flex items-center gap-2">
            <span className="w-3 h-3 rounded-full bg-red-500" />
            <span className="w-3 h-3 rounded-full bg-yellow-400" />
            <span className="w-3 h-3 rounded-full bg-green-500" />
            <span className="ml-4 text-xs text-slate-400 font-mono">
              aletheia verify
            </span>
          </div>

          {/* Terminal body */}
          <div className="bg-slate-900 px-5 py-5 font-mono text-sm leading-relaxed">
            <p className="text-slate-400">
              $ aletheia verify pr-42.aletheia.json
            </p>
            <p className="text-slate-300 mt-2">
              Loading evidence pack&hellip;
            </p>
            <p className="text-slate-300">Verifying hash chain&hellip;</p>
            <p className="text-green-400 mt-1">
              ✓ Chain intact — 23/23 events valid
            </p>
            <p className="text-slate-300 mt-1">
              Verifying Merkle root&hellip;
            </p>
            <p className="text-green-400">
              ✓ Merkle root matches — a3f9c12e…
            </p>
            <p className="text-slate-300 mt-1">
              Verifying Ed25519 signature&hellip;
            </p>
            <p className="text-green-400">
              ✓ Signature valid — key: 8b4d0e7f…
            </p>
            <p className="text-slate-300 mt-3">
              ─────────────────────────────────
            </p>
            <p className="text-green-400 font-bold mt-1">
              ✓ VERIFIED — evidence pack is authentic
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
