const SUMMARY_ITEMS = [
  { label: "Events", value: "23" },
  { label: "Chain", value: "23/23 valid" },
  { label: "Merkle Root", value: "a3f9c12e4b…" },
  { label: "Signatures", value: "1/1" },
] as const;

const TIMELINE_EVENTS = [
  {
    seq: 1,
    kind: "git.checkout",
    description: "Checked out branch feature/auth-refactor",
  },
  {
    seq: 7,
    kind: "llm.prompt",
    description: "Sent refactor prompt to GPT-4o — 1,204 tokens",
  },
  {
    seq: 23,
    kind: "git.commit",
    description: "Committed 14 files — SHA 8d3f9a1c",
  },
] as const;

export function EvidenceDemo() {
  return (
    <section className="bg-slate-50 py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-12">
          <h2 className="text-3xl sm:text-4xl font-bold text-slate-900">
            See what an evidence pack looks like
          </h2>
          <p className="mt-4 text-lg text-slate-600">
            A standalone, human-readable report — no external dependencies.
          </p>
        </div>

        {/* Mock evidence report card */}
        <div className="max-w-3xl mx-auto bg-white rounded-xl border border-slate-200 shadow-lg overflow-hidden">
          {/* Card header */}
          <div className="flex items-center justify-between px-6 py-4 border-b border-slate-200">
            <div>
              <h3 className="font-semibold text-slate-900 text-base">
                Evidence Report
              </h3>
              <p className="text-sm text-slate-500 font-mono mt-0.5">
                pr-42.aletheia.json
              </p>
            </div>
            <span className="inline-flex items-center px-3 py-1 rounded-full bg-green-100 text-green-700 text-xs font-bold tracking-wide">
              VERIFIED
            </span>
          </div>

          {/* Summary grid */}
          <div className="grid grid-cols-2 sm:grid-cols-4 gap-px bg-slate-100 border-b border-slate-200">
            {SUMMARY_ITEMS.map((item) => (
              <div key={item.label} className="bg-white px-5 py-4">
                <p className="text-xs text-slate-500 uppercase tracking-wide font-medium">
                  {item.label}
                </p>
                <p className="mt-1 text-base font-semibold text-slate-900 font-mono">
                  {item.value}
                </p>
              </div>
            ))}
          </div>

          {/* Mini timeline */}
          <div className="px-6 py-5">
            <h4 className="text-xs font-semibold uppercase tracking-wide text-slate-500 mb-4">
              Event Timeline
            </h4>
            <div className="flex flex-col gap-3">
              {TIMELINE_EVENTS.map((event) => (
                <div key={event.seq} className="flex items-start gap-4">
                  <span className="flex-shrink-0 w-7 h-7 rounded-full bg-brand-50 border border-brand-500 flex items-center justify-center text-xs font-bold text-brand-600">
                    {event.seq}
                  </span>
                  <div className="min-w-0">
                    <span className="text-xs font-mono font-semibold text-brand-600 bg-brand-50 px-2 py-0.5 rounded">
                      {event.kind}
                    </span>
                    <p className="text-sm text-slate-600 mt-1">
                      {event.description}
                    </p>
                  </div>
                </div>
              ))}
            </div>
          </div>

          {/* Footer caption */}
          <div className="px-6 py-3 bg-slate-50 border-t border-slate-100">
            <p className="text-xs text-slate-400 text-center">
              SHA-256 hash chain &middot; Merkle root &middot; Ed25519 signatures
              &middot; Standalone HTML export
            </p>
          </div>
        </div>
      </div>
    </section>
  );
}
