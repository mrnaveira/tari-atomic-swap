import * as React from 'react';
import AppBar from '@mui/material/AppBar';
import Container from '@mui/material/Container';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Toolbar from '@mui/material/Toolbar';
import { TariConnection, TariConnectorButton, TariPermissionAccountInfo, TariPermissionKeyList, TariPermissionTransactionGet, TariPermissionTransactionSend, TariPermissions } from 'tari-connector/src/index';
import { useNavigate } from "react-router-dom";

import Stack from '@mui/material/Stack';

import * as matchmaking from './tari-lib';
import { useEffect } from 'react';

export default function ConnectTari() {
  let signaling_server_address = import.meta.env.VITE_TARI_SIGNALING_SERVER_ADDRESS || "http://localhost:9100";

  const navigate = useNavigate();

  const [tari, setTari] = React.useState<TariConnection | undefined>();
  const [providers, setProviders] = React.useState([]);

  const goToSwap = (providers) => {
    navigate("/swap", { state: { providers } });
  }

  const onTariConnectButton = (tari: TariConnection) => {
		setTari(tari);
		window.tari = tari;
	};

  const setTariAnswer = async () => {
		tari?.setAnswer();
		await new Promise(f => setTimeout(f, 1000));
		window.tariConnected = true;
    let poviders = await matchmaking.get_all_provider_positions(tari);
    //console.log({poviders});
    setProviders(providers);
    goToSwap(poviders);
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
        </Toolbar>
      </AppBar>


      <Container maxWidth="sm">
        <Stack direction="row" alignItems="center" justifyContent="center"  sx={{ marginTop: 6 }}>
        <TariConnectorButton
						signalingServer={signaling_server_address}
						permissions={permissions}
						optional_permissions={optional_permissions}
						onOpen={onTariConnectButton}
					/>
					{tari ? <button onClick={async () => { await setTariAnswer(); }}>SetAnswer</button> : null}
        </Stack>
      </Container>

    </div>
  );
}
