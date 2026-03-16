import type { Config } from "tailwindcss";

const config: Config = {
  content: [
    "./app/**/*.{ts,tsx}",
    "./components/**/*.{ts,tsx}",
  ],
  theme: {
    extend: {
      colors: {
        brand: {
          50: "#eff6ff",
          500: "#2563eb",
          600: "#1d4ed8",
          700: "#1e40af",
        },
        success: "#16a34a",
      },
      fontFamily: {
        sans: ["Inter", "system-ui", "sans-serif"],
        mono: ["Fira Code", "Cascadia Code", "monospace"],
      },
    },
  },
  plugins: [],
};

export default config;
