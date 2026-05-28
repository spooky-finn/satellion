import { Alert, Stack } from '@mui/joy'
import { Component, type ReactNode } from 'react'
import { B, P } from '../shortcuts'

interface Props {
  children: ReactNode
  fallback?: (error: unknown, retry: () => void) => ReactNode
}

interface State {
  has_error: boolean
  error: unknown
}

export class ErrorBoundary extends Component<Props, State> {
  state: State = { has_error: false, error: undefined }

  static getDerivedStateFromError(error: unknown): State {
    return { has_error: true, error }
  }

  retry = () => this.setState({ has_error: false, error: undefined })

  render() {
    if (!this.state.has_error) return this.props.children
    if (this.props.fallback)
      return this.props.fallback(this.state.error, this.retry)
    return (
      <Alert color="danger" variant="soft">
        <Stack gap={1}>
          <P color="danger" level="body-sm">
            Something went wrong
          </P>
          <P level="body-xs">{describe(this.state.error)}</P>
          <B size="sm" variant="soft" onClick={this.retry}>
            Retry
          </B>
        </Stack>
      </Alert>
    )
  }
}

const describe = (e: unknown): string =>
  e instanceof Error ? e.message : String(e)
