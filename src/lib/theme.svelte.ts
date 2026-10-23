type Mode = 'light' | 'dark' | 'system'

/** Follows the system palette until the user saves an explicit preference. */
class Theme {
  private media = matchMedia('(prefers-color-scheme: dark)')
  private systemDark = $state(this.media.matches)
  private mode = $state<Mode>(this.savedMode())

  private savedMode(): Mode {
    // Keep the theme selected before the migration from Joy UI.
    const saved =
      localStorage.getItem('satellion-theme') ??
      localStorage.getItem('joy-mode')

    return saved === 'light' || saved === 'dark' ? saved : 'system'
  }

  get dark() {
    return this.mode === 'system' ? this.systemDark : this.mode === 'dark'
  }

  apply() {
    document.documentElement.classList.toggle('dark', this.dark)
    document.documentElement.style.colorScheme = this.dark ? 'dark' : 'light'
  }

  /** Apply the palette and return cleanup for the system theme listener. */
  watch() {
    const change = () => {
      this.systemDark = this.media.matches
      this.apply()
    }

    this.media.addEventListener('change', change)
    this.apply()

    return () => this.media.removeEventListener('change', change)
  }

  toggle() {
    this.mode = this.dark ? 'light' : 'dark'
    localStorage.setItem('satellion-theme', this.mode)
    this.apply()
  }
}

export const theme = new Theme()
