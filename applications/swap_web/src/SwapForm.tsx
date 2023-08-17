import * as React from 'react';
import AppBar from '@mui/material/AppBar';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Toolbar from '@mui/material/Toolbar';
import Button from '@mui/material/Button';
import Paper from '@mui/material/Paper';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import TextField from '@mui/material/TextField';
import ListItemText from '@mui/material/ListItemText';
import IconButton from '@mui/material/IconButton';
import AutorenewIcon from '@mui/icons-material/Autorenew';
import { TariConnection, TariConnectorButton, TariPermissionAccountInfo, TariPermissionKeyList, TariPermissionTransactionGet, TariPermissionTransactionSend, TariPermissions } from 'tari-connector/src/index';
import * as ReactDOM from 'react-dom';
import { useNavigate } from "react-router-dom";

import Stack from '@mui/material/Stack';
import Metamask from './Metamask';

import * as matchmaking from './tari-lib';
import { useEffect } from 'react';

export default function SwapForm() {
  let signaling_server_address = import.meta.env.VITE_TARI_SIGNALING_SERVER_ADDRESS || "http://localhost:9100";

  const navigate = useNavigate();

  const [tari, setTari] = React.useState<TariConnection | undefined>();
  const [isConnected, setIsConnected] = React.useState<boolean>(false);
  const [fromToken, setFromToken] = React.useState('eth.wei');
  const [fromTokenAmount, setFromTokenAmount] = React.useState(0);
  const [toToken, setToToken] = React.useState('tari');
  const [toTokenAmount, setToTokenAmount] = React.useState(0);
  const [providers, setProviders] = React.useState([]);
  const [bestSwap, setBestSwap] = React.useState(null);

  useEffect(() => {
    console.log("useEffect ", {fromToken, fromTokenAmount});
    updateBestSwap();
  }, [fromToken, fromTokenAmount]);  

  const onTariConnectButton = (tari: TariConnection) => {
		console.log("OnOpen");
		setTari(tari);
		window.tari = tari;
	};

  const setTariAnswer = async () => {
		tari?.setAnswer();
		await new Promise(f => setTimeout(f, 1000));
		setIsConnected(true);
		let res = await tari.sendMessage("keys.list", tari.token);

    let poviders = await matchmaking.get_all_provider_positions(tari);
    console.log({poviders});
    setProviders(providers);
	};

  const updateBestSwap = async () => {
    if(providers && providers.length === 0 && fromTokenAmount !=0 ) {
      console.log({fromToken, fromTokenAmount, toToken});
      let bestSwap = await matchmaking.get_best_match(tari, fromToken, fromTokenAmount, toToken);
      setBestSwap(bestSwap);
      console.log("updateBestSwap - ", bestSwap.expected_balance);
      setToTokenAmount(bestSwap.expected_balance);
    }
  }

  const handleFromToken = async (event) => {
    setFromToken(event.target.value);
    await updateBestSwap();
  };

  const handleToToken = async (event) => {
    setToToken(event.target.value);
    await updateBestSwap();
  };

  const handleFromTokenAmount = async (event) => {
    setFromTokenAmount(Number(event.target.value));
  };

  const beginSwap = async (event) => {
    event.preventDefault();

    // bestSwap.public_key = "0xCEe86979A65267229dF08E7a479E3CD097609de2"

    navigate("/steps", { state: { bestSwap, fromToken, fromTokenAmount, toToken } });
  };

  let permissions = new TariPermissions();
	permissions.addPermission(new TariPermissionAccountInfo())
	permissions.addPermission(
		new TariPermissionTransactionSend()
	);
	permissions.addPermission(
		new TariPermissionTransactionGet()
	);
  permissions.addPermission(
		new TariPermissionKeyList()
	);
	let optional_permissions = new TariPermissions();

  return (
    <div>

      <AppBar
        position="static"
        color="transparent"
        elevation={0}
      >
        <Toolbar sx={{ justifyContent: 'space-between' }}>
          <Stack direction='row'>
            <img width="35px" height="35px" src="/content/tari-logo.svg" />
            <Typography variant="h6" color="inherit" noWrap sx={{ mx: 1 }}>
              Tari Atomic Swap
            </Typography>
          </Stack>

          <Box>
          <TariConnectorButton
						signalingServer={signaling_server_address}
						permissions={permissions}
						optional_permissions={optional_permissions}
						onOpen={onTariConnectButton}
					/>
					{tari ? <button onClick={async () => { await setTariAnswer(); }}>SetAnswer</button> : null}
            </Box>

          <Metamask />
        </Toolbar>
      </AppBar>


      <Container maxWidth="sm">
        <Box sx={{ my: 10 }}>
          <Paper variant="outlined" elevation={0} sx={{ my: { xs: 3, md: 6 }, p: { xs: 2, md: 3 }, borderRadius: 8 }}>
            <Box sx={{ my: 1, mx: 1.5 }}>
              <Typography component="h1" variant="h6">
                Swap
              </Typography>
              <Stack direction="row" spacing={2} sx={{ marginTop: 4 }}>
                <Select
                  labelId="demo-simple-select-helper-label"
                  id="demo-simple-select-helper"
                  value={fromToken}
                  onChange={handleFromToken}
                  sx={{ width: '40%', borderRadius: 4 }}
                >
                  <MenuItem value="eth.wei">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/ethereum-logo.svg" />
                      <ListItemText primary="Ethereum" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="tari">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/tari-logo.svg" />
                      <ListItemText primary="Tari" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="minotari">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/minotari-logo.svg" />
                      <ListItemText primary="Minotari" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="bitcoin">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/bitcoin-logo.svg" />
                      <ListItemText primary="Bitcoin" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="monero">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/monero-logo.svg" />
                      <ListItemText primary="Monero" />
                    </Stack>
                  </MenuItem>
                </Select>
                <TextField sx={{ width: '60%' }} id="fromAmount" placeholder="0"
                  onChange={handleFromTokenAmount}
                  InputProps={{
                    sx: { borderRadius: 4 },
                  }}
                  inputProps={{
                    style: { textAlign: "right" },
                  }} />
              </Stack>
              <Stack direction="row-reverse" spacing={2} sx={{ marginTop: 1 }}>
                <Button variant="outlined" style={{ fontSize: 12, maxWidth: '20px', maxHeight: '20px', borderRadius: 8 }} >
                  MAX
                </Button>
                <Typography component="h1" style={{ fontSize: 12, maxHeight: '20px' }} >
                  Your balance is 0.00
                </Typography>
              </Stack>

              <Stack direction="row" alignItems="center" justifyContent="center">
                <IconButton aria-label="delete">
                  <AutorenewIcon />
                </IconButton>
              </Stack>

              <Stack direction="row" spacing={2} sx={{ marginTop: 2 }}>
                <Select
                  labelId="demo-simple-select-helper-label"
                  id="demo-simple-select-helper"
                  value={toToken}
                  onChange={handleToToken}
                  sx={{ width: '40%', borderRadius: 4 }}
                >
                  <MenuItem value="eth.wei">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/ethereum-logo.svg" />
                      <ListItemText primary="Ethereum" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="tari">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/tari-logo.svg" />
                      <ListItemText primary="Tari" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="minotari">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/minotari-logo.svg" />
                      <ListItemText primary="Minotari" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="bitcoin">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/bitcoin-logo.svg" />
                      <ListItemText primary="Bitcoin" />
                    </Stack>
                  </MenuItem>
                  <MenuItem value="monero">
                    <Stack direction="row" spacing={2}>
                      <img width="20px" height="20px" src="/content/monero-logo.svg" />
                      <ListItemText primary="Monero" />
                    </Stack>
                  </MenuItem>
                </Select>
                <TextField sx={{ width: '60%' }} id="toAmount" placeholder="0"
                  value={toTokenAmount}
                  InputProps={{
                    sx: { borderRadius: 4 },
                  }}
                  inputProps={{
                    style: { textAlign: "right" },
                  }} />
              </Stack>
              <Button sx={{ marginTop: 6, width: '100%', borderRadius: 4, fontSize: 20, textTransform: 'capitalize' }} variant="contained" onClick={beginSwap}>Begin Swap</Button>
            </Box>
          </Paper>
        </Box>
      </Container>

    </div>
  );
}
