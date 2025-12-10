/** @type {import('tailwindcss').Config} */
export default {
  content: [
    "./index.html",
    "./src/**/*.{js,ts,jsx,tsx}",
  ],
  theme: {
    extend: {
      screens: {
        '3xl': '1920px',  // Full HD / 1080p
        '4xl': '2560px',  // WQHD / 1440p
        '5xl': '3840px',  // 4K UHD
      },
    },
  },
  plugins: [],
}
