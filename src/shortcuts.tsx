import { Stack, StackProps, Typography } from '@mui/joy'

export const P = Typography
export const Row = (props: StackProps) => <Stack gap={1}  direction={'row'} {...props}/>