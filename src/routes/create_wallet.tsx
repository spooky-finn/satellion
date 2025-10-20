import { Button, Stack } from '@mui/joy'
import { P, Row } from '../shortcuts'

const CreateWallet = () => {
  return (
    <Stack gap={3} alignItems={'center'}>
      <P level='h2' color='primary'>Add wallet</P>
      <Row sx={{ width: 'min-content'}}>
        <Button>Import</Button>
        <Button>Generate</Button>
      </Row>
    </Stack>
  );
};

export default CreateWallet;