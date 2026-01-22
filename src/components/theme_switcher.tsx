import DarkModeIcon from '@mui/icons-material/DarkMode'
import LightModeIcon from '@mui/icons-material/LightMode'
import { Button } from '@mui/joy'
import { useColorScheme } from '@mui/joy/styles'
import React from 'react'

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
		<Button onClick={toggleTheme} variant="plain" size="sm" color="neutral">
			{mode === 'dark' ? <LightModeIcon /> : <DarkModeIcon />}
		</Button>
	)
}
