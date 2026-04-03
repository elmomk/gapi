/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        bg: "var(--color-bg)",
        surface: "var(--color-surface)",
        border: "var(--color-border)",
        text: "var(--color-text)",
        dim: "var(--color-dim)",
        accent: "var(--color-accent)",
        good: "var(--color-good)",
        warn: "var(--color-warn)",
        info: "var(--color-info)",
      },
      fontFamily: {
        mono: ["'SF Mono'", "'Fira Code'", "'JetBrains Mono'", "monospace"],
      },
    },
  },
  plugins: [],
}
