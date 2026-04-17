// hooks/useKeyboardRefetch.ts
import { useEffect } from 'react'

/**
 * Intercepts reload shortcuts and triggers a MobX action
 * @param action The MobX action to execute
 */
export const useKeyboardRefetch = (action: () => Promise<void>) => {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      const isR = event.key.toLowerCase() === 'r'
      const isModifier = event.metaKey || event.ctrlKey
      const isF5 = event.key === 'F5'

      if ((isModifier && isR) || isF5) {
        event.preventDefault()
        action()
      }
    }

    window.addEventListener('keydown', handleKeyDown)
    return () => window.removeEventListener('keydown', handleKeyDown)
  }, [action])
}
