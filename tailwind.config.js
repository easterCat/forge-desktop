/** @type {import('tailwindcss').Config} */
export default {
  content: [
    './index.html',
    './src/**/*.{vue,js,ts,jsx,tsx}',
  ],
  theme: {
    extend: {
      // Responsive breakpoints (design-notes §4.1)
      screens: {
        'sm': '480px',   // Narrow viewport / mobile
        'md': '768px',   // Tablet
        'lg': '1024px',  // Desktop
        'xl': '1280px',  // Large desktop
      },

      // Colors — reference CSS variables (src/assets/tokens/)
      colors: {
        // Glass layers (from CSS variables)
        'glass-window': 'var(--glass-window)',
        'glass-sidebar': 'var(--glass-sidebar)',
        'glass-topbar': 'var(--glass-topbar)',
        'glass-bg': 'var(--glass-bg)',
        'glass-bg-hover': 'var(--glass-bg-hover)',
        'glass-input': 'var(--glass-input)',
        'glass-input-focus': 'var(--glass-input-focus)',

        // Semantic colors
        'accent': 'var(--accent)',
        'accent-hover': 'var(--accent-hover)',
        'success': 'var(--success)',
        'error': 'var(--error)',
        'info': 'var(--info)',
        'warn': 'var(--warn)',

        // Text colors
        'fg': 'var(--fg)',
        'fg-muted': 'var(--fg-muted)',
        'fg-ghost': 'var(--fg-ghost)',
      },

      // Font families
      fontFamily: {
        sans: ['Inter', 'system-ui', '-apple-system', 'sans-serif'],
        mono: ['JetBrains Mono', 'SF Mono', 'Fira Code', 'monospace'],
      },

      // Border radius (from design tokens)
      borderRadius: {
        DEFAULT: 'var(--radius)',
        'sm': 'var(--radius-sm)',
        'lg': 'var(--radius-lg)',
        'xl': 'var(--radius-xl)',
      },

      // Spacing
      spacing: {
        'sidebar': 'var(--sidebar-w)',
        'topbar': 'var(--topbar-h)',
      },

      // Z-index
      zIndex: {
        'sidebar': 'var(--z-sidebar)',
        'topbar': 'var(--z-topbar)',
        'dropdown': 'var(--z-dropdown)',
        'modal-backdrop': 'var(--z-modal-backdrop)',
        'modal': 'var(--z-modal)',
        'toast': 'var(--z-toast)',
        'tooltip': 'var(--z-tooltip)',
      },

      // Animation durations (matching CSS tokens)
      transitionDuration: {
        'fast': '150ms',
        'base': '200ms',
        'slow': '300ms',
      },
    },
  },
  plugins: [],
}
