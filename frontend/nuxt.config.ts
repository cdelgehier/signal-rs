export default defineNuxtConfig({
  ssr: false,
  modules: ['@nuxt/ui'],
  css: ['~/assets/css/main.css'],
  ui: {
    colors: {
      primary: 'blue',
      neutral: 'zinc',
    },
  },
  compatibilityDate: '2025-05-19',
  future: { compatibilityVersion: 4 },
  app: {
    head: {
      title: 'Signal RS',
    },
  },
  vite: {
    // Prevent Vite from clearing the console on each reload in dev
    clearScreen: false,
    // Tauri expects a fixed port
    server: {
      port: 3000,
      strictPort: true,
    },
    envPrefix: ['VITE_', 'TAURI_'],
  },
})
