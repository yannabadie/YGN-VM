const STEPS = [
  {
    number: 1,
    title: "Capture",
    description:
      "Stream your agent's events into Aletheya. Each event is hashed and chained to the previous one in real time.",
    command: "git log --format=json | aletheia capture --session pr-42",
  },
  {
    number: 2,
    title: "Seal",
    description:
      "When the workflow completes, seal the session. Aletheya computes the Merkle root and signs the entire pack with your Ed25519 key.",
    command: "aletheia seal --session pr-42 --key signing.sec",
  },
  {
    number: 3,
    title: "Verify",
    description:
      "Anyone with the evidence pack and your public key can verify authenticity — no server required. Export as JSON, HTML, or Markdown.",
    command: "aletheia verify pr-42.aletheia.json",
  },
] as const;

export function HowItWorks() {
  return (
    <section id="how-it-works" className="py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-14">
          <h2 className="text-3xl sm:text-4xl font-bold text-slate-900">
            How it works
          </h2>
          <p className="mt-4 text-lg text-slate-600">
            Three commands. One tamper-evident evidence pack.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
          {STEPS.map((step) => (
            <div key={step.number} className="flex flex-col gap-4">
              {/* Numbered circle */}
              <div className="flex items-center gap-3">
                <div className="flex-shrink-0 w-10 h-10 rounded-full bg-brand-500 flex items-center justify-center text-white font-bold text-sm">
                  {step.number}
                </div>
                <h3 className="text-lg font-semibold text-slate-900">
                  {step.title}
                </h3>
              </div>

              <p className="text-slate-600 leading-relaxed text-sm">
                {step.description}
              </p>

              {/* Code block */}
              <div className="rounded-lg bg-slate-900 px-4 py-3">
                <code className="text-green-400 font-mono text-xs break-all">
                  $ {step.command}
                </code>
              </div>
            </div>
          ))}
        </div>
      </div>
    </section>
  );
}
