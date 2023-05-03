const primaryColor = "#2575FC";
const secondaryColor = "#A223CF";

module.exports = {
  content: ["./src/**/*.res"],
  safelist: [
    {
      pattern: /order-(1|2|3|4|5|6|7|9|10|11|12|first|last|none)/,
      variants: ["md"],
    },
  ],
  darkMode: "class",
  theme: {
    extend: {
      keyframes: {
        "curve-fade-in": {
          "0%": { opacity: "0" },
          "20%": { opacity: "0.4" },
          "100%": { opacity: "1" },
        },
      },
      animation: {
        "curve-fade-in": "curve-fade-in 5s",
      },
      screens: {
        nav: "1100px",
      },
      colors: {
        primary: primaryColor,
        secondary: secondaryColor,
      },
      borderColor: {
        DEFAULT: primaryColor,
      },
      width: {
        "1/6": "17%",
        "1/8": "12%",
        "1/10": "10%",
        "1/12": "8%",
        "1/16": "6%",
        "slightly-less-than-half": "45%",
        "30-percent": "30%",
        half: "50%",
        "9/10": "90%",
        "15/10": "150%",
        "price-width": "12rem",
        "mint-width": "38rem",
        "table-width": "50rem",
        "frame-width": "10rem",
        big: "28rem",
      },
      maxWidth: {
        "mint-width": "40rem",
        xxs: "15rem",
        big: "28rem",
        "50p": "50%",
      },
      margin: {
        "minus-12": "-3.4rem",
        "minus-quarter": "-25%",
        "minus-1": "-0.4rem",
      },
      inset: {
        half: "50%",
      },
      height: {
        "80-percent-screen": "80vh",
        "price-height": "6rem",
        big: "28rem",
        "70-percent-screen": "70vh",
        "60-percent-screen": "60vh",
        "50-percent-screen": "50vh",
      },
      boxShadow: {
        "inner-card": "inset 1px 1px 2px 0 rgba(0, 0, 0, 0.3)",
        "outer-card": "2px 2px 2px 0 rgba(0, 0, 0, 0.3)",
      },
      scale: {
        102: "1.02",
      },
      fontSize: {
        xxxxs: ".4rem",
        xxxs: ".5rem",
        xxs: ".6rem",
      },
      maxHeight: {
        "1/4": "25%",
        "1/2": "50%",
        "3/4": "75%",
        "9/10": "90%",
        "50-percent-screen": "50vh",
        "60-percent-screen": "60vh",
      },
      minWidth: {
        "1/2": "50%",
        "3/4": "75%",
        340: "340px",
        400: "400px",
        500: "500px",
        56: "56px",
        6: "1.5rem",
        md: "768px",
      },
      minHeight: {
        "half-screen": "50vh",
        "eighty-percent-screen": "80vh",
      },
      letterSpacing: {
        "btn-text": "0.2em",
      },
      order: {
        13: "13",
        14: "14",
        15: "15",
        16: "16",
        17: "17",
        18: "18",
      },
    },

    /* We override the default font-families with our own default prefs  */
    fontFamily: {
      sans: [
        "-apple-system",
        "BlinkMacSystemFont",
        "Helvetica Neue",
        "Arial",
        "sans-serif",
      ],
      serif: [
        "Georgia",
        "-apple-system",
        "BlinkMacSystemFont",
        "Helvetica Neue",
        "Arial",
        "sans-serif",
      ],
      mono: [
        "Menlo",
        "Monaco",
        "Consolas",
        "Roboto Mono",
        "SFMono-Regular",
        "Segoe UI",
        "Courier",
        "monospace",
      ],
      "font-name": ["font-name"],
      default: ["menlo", "'Roboto Mono'", "sans-serif"],
    },
  },
  plugins: [],
};
