/** @type {import('tailwindcss').Config} */
module.exports = {
  content: [
    "./index.html",
    "./src/**/*.rs",
  ],
  theme: {
    extend: {
      colors: {
        bg: "#08080f",
        bg2: "#0d0d1a",
        surface: "#141428",
        border: "#1e1e3a",
        text: "#e0e0e8",
        dim: "#5a5a7a",
        accent: "#00f0ff",
        green: "#00ff41",
        heart: "#ff2d78",
        sleep: "#bf5af2",
        stress: "#ff8c00",
        steps: "#00ff41",
        good: "#00ff41",
        warn: "#ff2d78",
        info: "#00f0ff",
      },
      fontFamily: {
        mono: ["'JetBrains Mono'", "'Fira Code'", "'SF Mono'", "monospace"],
      },
    },
  },
  plugins: [],
}
