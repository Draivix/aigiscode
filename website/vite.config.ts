import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import fs from 'fs';
import path from 'path';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [
    react(),
    tailwindcss(),
    {
      name: 'aigiscode-surface-endpoint',
      configureServer(server) {
        server.middlewares.use('/__aigiscode/surface', (_req, res) => {
          const surfacePath = path.resolve(__dirname, '../.aigiscode/architecture-surface.json');
          if (!fs.existsSync(surfacePath)) {
            res.statusCode = 404;
            res.setHeader('Content-Type', 'application/json');
            res.end(JSON.stringify({ error: 'architecture surface not found' }));
            return;
          }

          res.statusCode = 200;
          res.setHeader('Content-Type', 'application/json');
          res.end(fs.readFileSync(surfacePath, 'utf-8'));
        });
      },
    },
  ],
  resolve: {
    alias: {
      '@': path.resolve(__dirname, './src'),
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks: {
          'vendor-react': ['react', 'react-dom', 'react-router-dom'],
          'vendor-motion': ['motion'],
          'vendor-anime': ['animejs'],
          'vendor-i18n': ['i18next', 'react-i18next', 'i18next-browser-languagedetector'],
          'vendor-helmet': ['react-helmet-async'],
          'vendor-icons': ['@phosphor-icons/react'],
        },
      },
    },
  },
});
