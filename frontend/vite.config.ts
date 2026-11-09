import tailwindcss from '@tailwindcss/vite'
import react from '@vitejs/plugin-react'
import { defineConfig, loadEnv } from 'vite'

// 백엔드·compose와 같은 저장소 루트 .env(BACKEND_PORT·VITE_PORT)를 읽는다.
const rootDir = `${import.meta.dirname}/..`

export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, rootDir, '')
  return {
    envDir: rootDir,
    plugins: [react(), tailwindcss()],
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
