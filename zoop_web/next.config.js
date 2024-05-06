/** @type {import('next').NextConfig} */

const nextConfig = {
  reactStrictMode: false,
  // Note: This feature is required to use NextJS Image in SSG mode.
  // See https://nextjs.org/docs/messages/export-image-api for different workarounds.
  images: {
    unoptimized: true,
  },
  env: {
    zoopWebsocketServer: `ws://localhost:3000`,
    zoopHttpServer: `http://localhost:3000`,
    launcherHttpServer: `http://localhost:3000`,
  },
  async rewrites() {
    return [
      // Backend server
      {
        source: '/api/:path*',
        destination: 'http://localhost:8080/:path*' // Proxy to Backend
      },
      // Engine assets
      {
        source: '/room/assets/tire.png',
        destination: '/assets/tire.png'
      },
      {
        source: '/room/assets/car.png',
        destination: '/assets/car.png'
      },
      {
        source: '/room/assets/trace.png',
        destination: '/assets/trace.png'
      },
      {
        source: '/room/assets/building.glb',
        destination: '/assets/building.glb'
      },
    ]
  }
}

module.exports = nextConfig

