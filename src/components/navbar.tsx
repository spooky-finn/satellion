import { Link } from 'react-router'
import { route } from '../routes'
import { Row } from '../shortcuts'
import { ThemeSwitcher } from './theme_switcher'

export const Navbar = () => (
  <Row justifyContent={'space-between'}>
    <Row gap={3}>
      <Link to={route.bitcoin}>
        <img
          src={new URL('/bitcoin.webp', import.meta.url).toString()}
          alt="Bitcoin"
          width={32}
          height={32}
        />
      </Link>
      <Link to={route.ethereum}>
        <img
          src={new URL('/ethereum.webp', import.meta.url).toString()}
          alt="Ethereum"
          width={'auto'}
          height={32}
        />
      </Link>
    </Row>
    <ThemeSwitcher />
  </Row>
)
