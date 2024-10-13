import colors from "tailwindcss/colors";

/** @type {import('tailwindcss').Config} */
export default {
  content: ["./index.html", "./src/**/*.{js,ts,jsx,tsx,vue}"],
  theme: {
    colors: {
      ...colors,
      prime: {
        ...colors.yellow,
        50: "#ffffff",
      },
      back: {
        ...colors.stone,
        950: "#000000",
      },
      sec: {
        ...colors.stone,
      },
    },
  },
  plugins: [],
};
