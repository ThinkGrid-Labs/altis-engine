import { defineConfig } from 'vitepress';

export default defineConfig({
  title: 'Altis Engine',
  description:
    'Cloud-native Offer/Order engine for modern airline commerce — built in Rust, natively aligned with IATA Modern Retailing, NDC, and ONE Order.',
  base: '/altis-engine/',

  head: [
    [
      'meta',
      {
        name: 'og:title',
        content: 'Altis Engine — IATA Offer/Order Engine for Airline Retailing',
      },
    ],
    [
      'meta',
      {
        name: 'og:description',
        content:
          'Cloud-native airline retailing engine built in Rust. NDC v21.3, ONE Order, continuous pricing, Play Integrity, and IATA One Identity.',
      },
    ],
    ['meta', { name: 'og:type', content: 'website' }],
  ],

  themeConfig: {
    siteTitle: 'Altis Engine',

    nav: [
      { text: 'Guide', link: '/DEVELOPMENT' },
      { text: 'Architecture', link: '/architecture/OVERVIEW' },
      { text: 'API', link: '/API' },
      { text: 'Roadmap', link: '/ROADMAP' },
      {
        text: 'GitHub',
        link: 'https://github.com/ThinkGrid-Labs/altis-engine',
      },
    ],

    sidebar: [
      {
        text: 'Getting Started',
        items: [
          { text: 'Development Guide', link: '/DEVELOPMENT' },
          { text: 'Usage', link: '/USAGE' },
          { text: 'Deployment', link: '/DEPLOYMENT' },
        ],
      },
      {
        text: 'Architecture',
        items: [
          { text: 'Overview', link: '/architecture/OVERVIEW' },
          {
            text: 'AI Brain',
            link: '/architecture/AI_BRAIN_TECHNICAL',
          },
          {
            text: 'AI Integration',
            link: '/architecture/ai_integration',
          },
          {
            text: 'Session Management',
            link: '/architecture/session_management',
          },
          {
            text: 'Deployment Strategy',
            link: '/architecture/DEPLOYMENT_STRATEGY',
          },
        ],
      },
      {
        text: 'Domain Concepts',
        items: [
          { text: 'Offer & Order', link: '/OFFER_ORDER' },
          { text: 'Pricing', link: '/PRICING' },
          { text: 'Offer Customization', link: '/OFFER_CUSTOMIZATION' },
          { text: 'Seat Hold', link: '/SEAT_HOLD' },
          { text: 'Customer Journey', link: '/CUSTOMER_JOURNEY' },
          { text: 'Business Rules', link: '/BUSINESS_RULES' },
        ],
      },
      {
        text: 'Reference',
        items: [
          { text: 'API Reference', link: '/API' },
          { text: 'Admin CMS', link: '/ADMIN_CMS' },
          { text: 'IATA Alignment Roadmap', link: '/IATA_ALIGNMENT_ROADMAP' },
          { text: 'Roadmap', link: '/ROADMAP' },
        ],
      },
    ],

    socialLinks: [
      {
        icon: 'github',
        link: 'https://github.com/ThinkGrid-Labs/altis-engine',
      },
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © ThinkGrid Labs',
    },

    search: {
      provider: 'local',
    },

    editLink: {
      pattern:
        'https://github.com/ThinkGrid-Labs/altis-engine/edit/main/docs/:path',
      text: 'Edit this page on GitHub',
    },
  },
});
