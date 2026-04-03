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
        bg2: "var(--color-bg2)",
        surface: "var(--color-surface-solid)",
        border: "var(--color-border)",
        text: "var(--color-text)",
        dim: "var(--color-dim)",
        accent: "var(--color-accent)",
        heart: "var(--color-heart)",
        sleep: "var(--color-sleep)",
        stress: "var(--color-stress)",
        steps: "var(--color-steps)",
        good: "var(--color-good)",
        warn: "var(--color-warn)",
        info: "var(--color-info)",
      },
      fontFamily: {
        display: ["'Inter'", "system-ui", "-apple-system", "sans-serif"],
        mono: ["'SF Mono'", "'Fira Code'", "'JetBrains Mono'", "monospace"],
      },
      animation: {
        'fade-in': 'fadeIn 0.3s ease-out',
        'slide-up': 'slideUp 0.3s ease-out',
      },
      keyframes: {
        fadeIn: { '0%': { opacity: '0' }, '100%': { opacity: '1' } },
        slideUp: { '0%': { opacity: '0', transform: 'translateY(8px)' }, '100%': { opacity: '1', transform: 'translateY(0)' } },
      },
    },
  },
  plugins: [],
}
