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

import Stack from '@mui/material/Stack';

export default function App() {
  let signaling_server_address = import.meta.env.VITE_TARI_SIGNALING_SERVER_ADDRESS || "http://localhost:9100";
	let tari_lp_index: string = import.meta.env.VITE_TARI_LP_INDEX;

  const [tari, setTari] = React.useState<TariConnection | undefined>();
  const [isConnected, setIsConnected] = React.useState<boolean>(false);
  const [fromToken, setFromToken] = React.useState('eth');
  const [toToken, setToToken] = React.useState('tari');

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
    console.log({res});
	};

  const handleFromToken = (event) => {
    setFromToken(event.target.value);
  };

  const handleToToken = (event) => {
    setToToken(event.target.value);
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

          <Button variant="contained" sx={{ my: 1, mx: 1.5, borderRadius: 8, textTransform: 'none' }}>
            Connect Tari Wallet
          </Button>
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
                  <MenuItem value="eth">
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
                  <MenuItem value="eth">
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
                  InputProps={{
                    sx: { borderRadius: 4 },
                  }}
                  inputProps={{
                    style: { textAlign: "right" },
                  }} />
              </Stack>
              <Button sx={{ marginTop: 6, width: '100%', borderRadius: 4, fontSize: 20, textTransform: 'capitalize' }} variant="contained">Begin Swap</Button>
            </Box>
          </Paper>
        </Box>
      </Container>

    </div>
  );
}
