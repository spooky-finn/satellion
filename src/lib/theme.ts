import { extendTheme } from '@mui/joy/styles'

export const theme = extendTheme({
  cssVarPrefix: 'mode-toggle',
  // @ts-expect-error
  colorSchemeSelector: '.demo_mode-toggle-%s',
  colorSchemes: {
    dark: {
      palette: {
        primary: {
          50: '#fff8e7',
          100: '#fdedc8',
          200: '#fbd98e',
          300: '#f8c154',
          400: '#f5a623',
          500: '#e8900a',
          600: '#c47408',
          700: '#9a5a06',
          800: '#704104',
          900: '#4a2b02',
          solidBg: '#e8900a',
          solidHoverBg: '#c47408',
          solidActiveBg: '#9a5a06',
          softBg: 'rgba(232, 144, 10, 0.15)',
          softHoverBg: 'rgba(232, 144, 10, 0.25)',
          softColor: '#f8c154',
          outlinedColor: '#f5a623',
          outlinedBorder: 'rgba(232, 144, 10, 0.45)',
          outlinedHoverBg: 'rgba(232, 144, 10, 0.1)',
          plainColor: '#f5a623',
          plainHoverBg: 'rgba(232, 144, 10, 0.1)',
        },
        neutral: {
          50: '#f5f0eb',
          100: '#e8e0d5',
          200: '#d0c4b5',
          300: '#b5a594',
          400: '#9a8878',
          500: '#7d6e60',
          600: '#61554a',
          700: '#473e35',
          800: '#302a23',
          900: '#1c1814',
          solidBg: '#47403a',
          solidHoverBg: '#5a524b',
          softBg: 'rgba(255, 235, 200, 0.08)',
          softHoverBg: 'rgba(255, 235, 200, 0.14)',
          softColor: '#c4b5a8',
          outlinedBorder: 'rgba(200, 180, 160, 0.28)',
          plainHoverBg: 'rgba(255, 235, 200, 0.08)',
          plainColor: '#c4b5a8',
        },
        background: {
          body: '#141210',
          surface: '#1c1916',
          popup: '#241f1b',
          level1: '#241f1b',
          level2: '#2e2822',
          level3: '#3a332c',
        },
        text: {
          primary: '#f0e8dd',
          secondary: '#c4b5a8',
          tertiary: '#8a7a6e',
          icon: '#8a7a6e',
        },
        divider: 'rgba(200, 175, 150, 0.14)',
        focusVisible: 'rgba(232, 144, 10, 0.5)',
      },
    },
  },
  components: {
    JoyButton: {
      defaultProps: {
        size: 'sm',
        sx: {
          width: 'fit-content',
        },
      },
    },
    JoyInput: {
      defaultProps: {
        autoComplete: 'off',
      },
    },
  },
})
