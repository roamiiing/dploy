import { defineConfig } from "vitepress";

export default defineConfig({
  title: "dploy",
  description: "The easiest way to deploy your applications",
  srcDir: "src",

  vite: {
    publicDir: "../public",
  },

  themeConfig: {
    nav: [
      { text: "Home", link: "/" },
      { text: "Getting Started", link: "/getting-started" },
    ],

    sidebar: [
      {
        text: "Getting started",
        link: "/getting-started",
      },
      {
        text: "Important considerations",
        link: "/important-considerations",
      },
    ],

    socialLinks: [
      { icon: "github", link: "https://github.com/roamiiing/dploy" },
    ],
  },
});
