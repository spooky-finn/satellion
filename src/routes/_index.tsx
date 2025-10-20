import { invoke } from '@tauri-apps/api/core'
import { useEffect } from 'react'
import { useNavigate } from 'react-router'

const Home = () => {
  const navigate = useNavigate();
  
  useEffect(() => {
    invoke('wallet_exists').then((exists) => {
      if (!exists) {
        navigate('/create_wallet')
      }
    })
  }, [])

  return (
    <div>
      <h1>Satellion Wallet</h1>
    </div>
  );
};

export default Home;
