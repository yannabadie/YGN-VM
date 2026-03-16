import { PRICING } from "@/lib/constants";

const plans = [PRICING.sprint, PRICING.selfHosted] as const;

export function Pricing() {
  return (
    <section id="pricing" className="py-20 px-4 sm:px-6 lg:px-8">
      <div className="max-w-6xl mx-auto">
        <div className="text-center mb-14">
          <h2 className="text-3xl sm:text-4xl font-bold text-slate-900">
            Simple, transparent pricing
          </h2>
          <p className="mt-4 text-lg text-slate-600">
            Choose the plan that fits your workflow.
          </p>
        </div>

        <div className="grid grid-cols-1 md:grid-cols-2 gap-8 max-w-3xl mx-auto">
          {plans.map((plan) => (
            <div
              key={plan.name}
              className={[
                "relative flex flex-col rounded-xl border p-8 shadow-sm",
                plan.highlighted
                  ? "border-brand-500 ring-2 ring-brand-500 ring-offset-2"
                  : "border-slate-200",
              ].join(" ")}
            >
              {/* Recommended badge */}
              {plan.highlighted && (
                <div className="absolute -top-3.5 left-1/2 -translate-x-1/2">
                  <span className="inline-flex items-center px-3 py-1 rounded-full bg-brand-500 text-white text-xs font-bold tracking-wide shadow">
                    Recommended
                  </span>
                </div>
              )}

              {/* Plan header */}
              <div className="mb-6">
                <h3 className="text-lg font-semibold text-slate-900">
                  {plan.name}
                </h3>
                <p className="mt-1 text-sm text-slate-500">{plan.description}</p>
              </div>

              {/* Price */}
              <div className="flex items-end gap-1 mb-6">
                <span className="text-4xl font-bold text-slate-900">
                  {plan.price}
                </span>
                <span className="text-slate-500 text-sm mb-1">{plan.period}</span>
              </div>

              {/* Features list */}
              <ul className="flex flex-col gap-3 mb-8 flex-1">
                {plan.features.map((feature) => (
                  <li key={feature} className="flex items-start gap-3">
                    <svg
                      className="flex-shrink-0 w-4 h-4 text-success mt-0.5"
                      viewBox="0 0 16 16"
                      fill="currentColor"
                      aria-hidden="true"
                    >
                      <path
                        fillRule="evenodd"
                        d="M12.78 4.22a.75.75 0 010 1.06l-5.5 5.5a.75.75 0 01-1.06 0l-2.5-2.5a.75.75 0 011.06-1.06L6.75 9.19l4.97-4.97a.75.75 0 011.06 0z"
                      />
                    </svg>
                    <span className="text-sm text-slate-600">{feature}</span>
                  </li>
                ))}
              </ul>

              {/* CTA */}
              <a
                href={plan.link}
                target="_blank"
                rel="noopener noreferrer"
                className={[
                  "inline-flex items-center justify-center w-full px-6 py-3 rounded-lg font-semibold text-sm transition-colors",
                  plan.highlighted
                    ? "bg-brand-500 text-white hover:bg-brand-600 active:bg-brand-700"
                    : "border border-slate-300 text-slate-700 hover:border-slate-400 hover:bg-slate-50",
                ].join(" ")}
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
