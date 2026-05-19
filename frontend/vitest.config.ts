import { defineConfig } from 'vitest/config'
import { defineVitestProject } from '@nuxt/test-utils/config'

export default defineConfig({
  test: {
    projects: [
      // Tests unitaires purs (DDD : entités, schemas, use-cases) — pas besoin de Nuxt
      {
        test: {
          name: 'unit',
          environment: 'happy-dom',
          include: ['app/**/*.spec.ts'],
        },
      },
      // Tests composants Nuxt (composables, pages) — avec environment Nuxt
      await defineVitestProject({
        test: {
          name: 'nuxt',
          environment: 'nuxt',
          include: ['test/nuxt/**/*.nuxt.spec.ts'],
        },
      }),
    ],
  },
})
