import * as React from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';

export default function Withdraw(props) {
  console.log(props.swapDetails);
  console.log(props.ongoingSwap);

  const provider_address = props.swapDetails.bestSwap.network_address;
  const expected_balance = props.swapDetails.bestSwap.expected_balance;
  const toToken = props.swapDetails.toToken;
  const contract_id_provider = props.ongoingSwap.contract_id_provider;
  const swap_id = props.ongoingSwap.swap_id;
  const preimage = props.ongoingSwap.preimage;

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

  const withdraw = async () => {
    await withdrawProviderFunds(contract_id_provider, toToken, preimage);
    await pushPreimageToprovider(contract_id_provider, swap_id, preimage);

    props.onCompletion(props.ongoingSwap);
  }

  const withdrawProviderFunds = async (contract_id_provider, toToken, preimage) => {
    // switch on the "toToken" and submit a transaction to the targeted network
    // should return and error if the contract does not exist or does not have the requested amount
  }

  const pushPreimageToprovider = async (provider_address, swap_id, preimage) => {
  }
  
  return (
    <div>
      <React.Fragment>
        <Stack direction="row" alignItems="center" justifyContent="center" sx={{ marginTop: 2 }}>
          <Typography variant="body1">The provider has locked <Box component="span" fontWeight='fontWeightMedium'>{expected_balance} {getTokenName(toToken)}</Box> for you. Please proceed to withdraw the funds</Typography>
        </Stack>
        <Box sx={{ display: 'flex', justifyContent: 'center' }}>
          <Button
            variant="contained"
            onClick={withdraw}
            sx={{  mt: 3, ml: 1, borderRadius: 8, textTransform: 'none' }} 
          > Withdraw
          </Button>
        </Box>
      </React.Fragment>
    </div>
  );
}
