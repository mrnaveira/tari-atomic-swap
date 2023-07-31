import * as React from 'react';
import { createTheme, ThemeProvider } from '@mui/material/styles';
import AppBar from '@mui/material/AppBar';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Toolbar from '@mui/material/Toolbar';
import Button from '@mui/material/Button';
import Paper from '@mui/material/Paper';
import FormControl from '@mui/material/FormControl';
import InputLabel from '@mui/material/InputLabel';
import Select from '@mui/material/Select';
import MenuItem from '@mui/material/MenuItem';
import FormHelperText from '@mui/material/FormHelperText';
import TextField from '@mui/material/TextField';
import ListItemIcon from '@mui/material/ListItemIcon';
import ListItemText from '@mui/material/ListItemText';
import Icon from '@mui/material/Icon';
import IconButton from '@mui/material/IconButton';
import AutorenewIcon from '@mui/icons-material/Autorenew';
import Link from '@mui/icons-material/Autorenew';

import Stack from '@mui/material/Stack';

export default function App() {
  const [fromToken, setFromToken] = React.useState('eth');
  const [toToken, setToToken] = React.useState('tari');

  const handleFromToken = (event) => {
    setFromToken(event.target.value);
  };

  const handleToToken = (event) => {
    setToToken(event.target.value);
  };

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

          <nav>
            <Button
              variant="button"
              color="text.primary"
              href="#"
              sx={{ my: 1, mx: 1.5, textTransform: 'none' }}
            >
              Swap
            </Button>
            <Button
              variant="button"
              color="text.primary"
              href="#"
              sx={{ my: 1, mx: 1.5, textTransform: 'none' }}
            >
              Liquidity
            </Button>
            <Button
              variant="button"
              color="text.primary"
              href="#"
              sx={{ my: 1, mx: 1.5, textTransform: 'none' }}
            >
              Analytics
            </Button>
          </nav>
          <Button href="#" variant="contained" sx={{ my: 1, mx: 1.5, borderRadius: 8, textTransform: 'none' }}>
            Connect wallet
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
              <Button sx={{ marginTop: 6, width: '100%', borderRadius: 4, fontSize: 20, textTransform: 'capitalize' }} variant="contained">Swap</Button>
            </Box>
          </Paper>
        </Box>
      </Container>
    </div>
  );
}
