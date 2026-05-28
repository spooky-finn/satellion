import DarkModeIcon from '@mui/icons-material/DarkMode'
import LightModeIcon from '@mui/icons-material/LightMode'
import { useColorScheme } from '@mui/joy/styles'
import React from 'react'
import { B } from '../shortcuts'

export function ThemeSwitcher() {
  const { mode, setMode } = useColorScheme()
  const [mounted, setMounted] = React.useState(false)

  React.useEffect(() => {
    setMounted(true)
  }, [])

  if (!mounted) {
    return null
  }

  const toggleTheme = () => {
    setMode(mode === 'light' ? 'dark' : 'light')
  }

  return (
    <B onClick={toggleTheme} variant="plain" size="sm" color="neutral">
      {mode === 'dark' ? <LightModeIcon /> : <DarkModeIcon />}
    </B>
  )
}
