import { Alert, Box, Container, Stack } from '@mui/joy'
import { observer } from 'mobx-react-lite'
import { notifier } from './notifier'

export const NotifierOverlay = observer(() => {
  return (
    <Box
      sx={{
        position: 'fixed',
        bottom: 10,
        right: 10,
        zIndex: 1000
      }}
    >
      <Stack gap={1}>
        {notifier
          .all()
          .toReversed()
          .map(each => (
            <Alert
              key={each.id}
              variant="solid"
              size="sm"
              color={each.level === 'err' ? 'danger' : 'success'}
            >
              <Container maxWidth="sm">{each.msg}</Container>
            </Alert>
          ))}
      </Stack>
    </Box>
  )
})
