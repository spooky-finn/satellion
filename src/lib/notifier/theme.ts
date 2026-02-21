import { extendTheme } from '@mui/joy/styles'

export const theme = extendTheme({
  cssVarPrefix: 'mode-toggle',
  // @ts-expect-error
  colorSchemeSelector: '.demo_mode-toggle-%s',
  components: {
    JoyButton: {
      defaultProps: {
        size: 'sm',
      },
    },
    JoyInput: {
      defaultProps: {
        autoComplete: 'off',
      },
    },
  },
})
