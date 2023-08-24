import * as React from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';
import Stack from '@mui/material/Stack';
import { formatEther } from 'ethers/lib/utils';

export default function LockFunds(props) {
  console.log(props.swapDetails);
  console.log(props.ongoingSwap);

  const expected_balance = props.swapDetails.bestSwap.expected_balance;
  const fromToken = props.swapDetails.fromToken;
  const fromTokenAmount = props.swapDetails.fromTokenAmount;
  const toToken = props.swapDetails.toToken;

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

  function formatTokenAmount(token, amount) {
    switch (token) {
      case "eth.wei":
        return formatEther(amount);
      case "tari":
        return amount;
      default:
        0;
    }
  }

  return (
    <div>
      <React.Fragment>
        <Stack direction="row" alignItems="center" justifyContent="center" sx={{ marginTop: 2 }}>
          <Typography variant="body1">You succesfully exchanged <Box component="span" fontWeight='fontWeightMedium'>{formatTokenAmount(fromToken, fromTokenAmount)} {getTokenName(fromToken)}</Box> for <Box component="span" fontWeight='fontWeightMedium'>{expected_balance} {getTokenName(toToken)}</Box></Typography>
        </Stack>
        <Box sx={{ display: 'flex', justifyContent: 'center' }}>
          <Button
            variant="contained"
            onClick={props.onCompletion}
            sx={{  mt: 3, ml: 1, borderRadius: 8, textTransform: 'none' }} 
          > Go to main page
          </Button>
        </Box>
      </React.Fragment>
    </div>
  );
}
