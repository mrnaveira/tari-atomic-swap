import * as React from 'react';
import Typography from '@mui/material/Typography';
import Box from '@mui/material/Box';
import Button from '@mui/material/Button';
import SyncAltIcon from '@mui/icons-material/SyncAlt';
import List from '@mui/material/List';
import ListItem from '@mui/material/ListItem';
import ListItemText from '@mui/material/ListItemText';
import Stack from '@mui/material/Stack';
import sha256 from 'crypto-js/sha256';
import * as cryptoEncHex from 'crypto-js/enc-hex';
import { v4 as uuidv4 } from 'uuid';

export default function LockFunds(props) {
  console.log(props.swapDetails);

  const fromToken = props.swapDetails.fromToken;
  const fromTokenAmount = props.swapDetails.fromTokenAmount;
  const toToken = props.swapDetails.toToken;
  const toTokenAmount = props.swapDetails.bestSwap.expected_balance;
  const provider_address = props.swapDetails.bestSwap.network_address;
  const provider_key = props.swapDetails.bestSwap.public_key;

  function getTokenName(token) {
    switch (token) {
      case "eth.wei":
        return "ether";
      case "tari":
        return "tari";
      default:
        "";
    }
  }

  function getSwapDetails() {
    return (
      <Typography variant="body2">
        {fromTokenAmount} {getTokenName(fromToken)}  <SyncAltIcon sx={{ fontSize: "small", verticalAlign: 'middle' }} /> {toTokenAmount} {getTokenName(toToken)}
      </Typography>
    );
  }

  const lockFunds = async () => {
    console.log("lockFunds");

    let preimage = createPreimage();
    let hashlock = createHashlock(preimage);

    let swap_id = await requestSwapFromProvider(hashlock);
    
    let contract_id_user = await publishLockContract(provider_key, fromToken, fromTokenAmount, hashlock);
    let contract_id_provider = await requestLockFundsFromProvider(swap_id, contract_id_user);
    
    // go to the next step in the process
    let payload = {
      swap_id,
      preimage,
      hashlock,
      contract_id_user,
      contract_id_provider,
    };
    props.onCompletion(payload);
  };

  const createPreimage = () => {
    // safe random value
    return uuidv4();
  }

  const createHashlock = (preimage) => {
    // depending on the network, we may need to use a different hash algorithm
    return sha256(preimage).toString(cryptoEncHex);
  }

  const requestSwapFromProvider = async (provider_address, hashlock) => {
    // proposal
    // client_address: String,
    // hashlock: Hashlock,
    // position: Position
        // provided_token
        // provided_token_balance
        // requested_token
        // requested_token_balance

    // swap_id
    let swap_id = '123-123';

    return swap_id;
  }

  const publishLockContract = async (provider_key, fromToken, fromTokenAmount, hashlock) => {
    // switch on the "fromToken" and submit a transaction to the targeted network
    // the return will be an identifier of the deployed lock contract in the target network
    let contract_id = "contract_id_user";
    return contract_id;
  }

  const requestLockFundsFromProvider = async (provider_address, swap_id, contract_id_user) => {
    // request
    // swap_id: String,
    // contract_id: ContractId,

    // contract_id
    let contract_id_provider = 'contract_id_provider';

    return contract_id_provider;
  }

  return (
    <div>
      <React.Fragment>
        <List disablePadding>
          <ListItem sx={{ py: 1, px: 0 }}>
            <ListItemText primary="Swap details" />
            {getSwapDetails()}
          </ListItem>
          <ListItem sx={{ py: 1, px: 0 }}>
            <ListItemText primary="Liquidity provider address" />
            <Typography variant="body2">{provider_address}</Typography>
          </ListItem>
          <ListItem sx={{ py: 1, px: 0 }}>
            <ListItemText primary="Liquidity provider public key" />
            <Typography variant="body2">{provider_key}</Typography>
          </ListItem>
        </List>
        
        <Stack direction="row" alignItems="center" justifyContent="center" sx={{ marginTop: 4 }}>
          <Typography variant="body1">You need to lock <Box component="span" fontWeight='fontWeightMedium'>{fromTokenAmount} {getTokenName(fromToken)}</Box> to begin the atomic swap</Typography>
        </Stack>

        <Box sx={{ display: 'flex', justifyContent: 'center' }}>
          <Button
            variant="contained"
            onClick={lockFunds}
            sx={{ mt: 3, ml: 1, borderRadius: 8, textTransform: 'none' }}
          > Lock funds
          </Button>
        </Box>
      </React.Fragment>
    </div>
  );
}
