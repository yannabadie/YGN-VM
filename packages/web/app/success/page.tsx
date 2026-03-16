"use client";

import { Suspense } from "react";
import { useSearchParams } from "next/navigation";
import Link from "next/link";

function SuccessContent() {
  const searchParams = useSearchParams();
  const plan = searchParams.get("plan");

  let message = "We'll be in touch shortly.";
  if (plan === "sprint") {
    message =
      "We'll reach out within 24 hours to schedule your Evidence Sprint.";
  } else if (plan === "self-hosted") {
    message = "Check your email for setup instructions.";
  }

  return (
    <div className="text-center max-w-md px-6">
      <div className="flex items-center justify-center w-16 h-16 mx-auto mb-6 rounded-full bg-green-100">
        <svg
          className="w-8 h-8 text-green-600"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
          aria-hidden="true"
        >
          <path
            strokeLinecap="round"
            strokeLinejoin="round"
            strokeWidth={2}
            d="M5 13l4 4L19 7"
          />
        </svg>
      </div>
      <h1 className="text-3xl font-bold text-gray-900 mb-4">Thank you!</h1>
      <p className="text-lg text-gray-600 mb-8">{message}</p>
      <Link
        href="/"
        className="inline-block px-6 py-3 bg-blue-600 text-white font-semibold rounded-lg hover:bg-blue-700 transition-colors"
      >
        Back to Home
      </Link>
    </div>
  );
}

export default function SuccessPage() {
  return (
    <div className="min-h-screen flex items-center justify-center bg-white">
      <Suspense
        fallback={
          <div className="text-center">
            <p className="text-gray-600">Loading...</p>
          </div>
        }
      >
        <SuccessContent />
      </Suspense>
    </div>
  );
}
