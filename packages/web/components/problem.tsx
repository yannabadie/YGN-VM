const CARDS = [
  {
    icon: "🔒",
    title: "Logs aren't proof",
    description:
      "Traditional logs can be modified after the fact. Without cryptographic guarantees, there is no way to prove an agent did — or didn't — take a specific action.",
  },
  {
    icon: "📋",
    title: "Compliance demands evidence",
    description:
      "SOC 2, ISO 27001, and the EU AI Act require more than screenshots. Auditors need tamper-evident, signed artefacts that stand up to scrutiny.",
  },
  {
    icon: "⚡",
    title: "Agents act fast, humans audit slow",
    description:
      "AI agents execute hundreds of actions per minute. Cryptographic receipts let you reconstruct and verify exactly what happened — days or months later.",
  },
] as const;

export function Problem() {
  return (
    <section className="bg-slate-50 py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-14">
          <h2 className="text-3xl sm:text-4xl font-bold text-slate-900">
            Why evidence packs?
          </h2>
          <p className="mt-4 text-lg text-slate-600">
            Monitoring tells you what happened. Aletheya proves it.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
          {CARDS.map((card) => (
            <div
              key={card.title}
              className="bg-white rounded-xl border border-slate-200 p-8 shadow-sm hover:shadow-md transition-shadow"
            >
              <div className="text-3xl mb-4">{card.icon}</div>
              <h3 className="text-lg font-semibold text-slate-900 mb-3">
                {card.title}
              </h3>
              <p className="text-slate-600 leading-relaxed text-sm">
                {card.description}
              </p>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
