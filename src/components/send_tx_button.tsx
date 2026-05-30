import SendIcon from '@mui/icons-material/Send'
import { Box, Button, type ButtonProps } from '@mui/joy'
import { useEffect, useRef, useState } from 'react'

interface Props extends Omit<ButtonProps, 'onClick'> {
  onSend: () => void
  holdMs?: number
}

export const SendTxButton = ({
  onSend,
  holdMs = 3000,
  children = 'Hold to send',
  disabled,
  sx,
  ...rest
}: Props) => {
  const [holding, setHolding] = useState(false)
  const timer = useRef<ReturnType<typeof setTimeout> | null>(null)

  useEffect(
    () => () => {
      if (timer.current) clearTimeout(timer.current)
    },
    [],
  )

  const start = () => {
    if (disabled) return
    setHolding(true)
    timer.current = setTimeout(() => {
      timer.current = null
      setHolding(false)
      onSend()
    }, holdMs)
  }

  const cancel = () => {
    if (timer.current) {
      clearTimeout(timer.current)
      timer.current = null
    }
    setHolding(false)
  }

  return (
    <Button
      {...rest}
      disabled={disabled}
      onPointerDown={start}
      onPointerUp={cancel}
      onPointerLeave={cancel}
      onPointerCancel={cancel}
      color="success"
      sx={{
        position: 'relative',
        overflow: 'hidden',
        width: 'fit-content',
        ...sx,
      }}
      endDecorator={<SendIcon />}
    >
      <Box
        sx={{
          position: 'absolute',
          left: 0,
          top: 0,
          bottom: 0,
          width: holding ? '100%' : '0%',
          bgcolor: 'rgba(255,255,255,0.25)',
          transition: holding
            ? `width ${holdMs}ms linear`
            : 'width 150ms ease-out',
          pointerEvents: 'none',
        }}
      />
      <Box component="span" sx={{ position: 'relative', zIndex: 1 }}>
        {children}
      </Box>
    </Button>
  )
}
