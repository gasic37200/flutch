import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { readFile } from 'node:fs/promises'
import { defineConfig, loadEnv } from 'vite'

// 백엔드·compose와 같은 저장소 루트 .env(BACKEND_PORT·VITE_PORT)를 읽는다.
const rootDir = `${import.meta.dirname}/..`

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, rootDir, '')
  return {
    envDir: rootDir,
    plugins: [
      react(),
      tailwindcss(),
      {
        // 운영은 nginx가 /media/thumbs를 내준다. 개발에서는 백엔드가 쓰는 폴더를 그대로 읽어 준다.
        name: 'dev-thumbs',
        configureServer(server) {
          server.middlewares.use('/media/thumbs', (req, res, next) => {
            const name = (req.url ?? '').split('?')[0].replace(/^\//, '')
            if (!/^\d+\.jpg$/.test(name)) return next()
            readFile(`${rootDir}/backend/thumbs/${name}`).then(
              (buf) => res.setHeader('Content-Type', 'image/jpeg').end(buf),
              () => next(),
            )
          })
        },
      },
    ],
    server: {
      port: Number(env.VITE_PORT || 5173),
      strictPort: true,
      // Origin 헤더를 그대로 넘겨야 백엔드 Origin 검사(ALLOWED_ORIGINS)를 통과한다(changeOrigin 끔).
      proxy: {
        '/api': { target: `http://localhost:${env.BACKEND_PORT || 8080}`, ws: true },
      },
    },
  }
})
