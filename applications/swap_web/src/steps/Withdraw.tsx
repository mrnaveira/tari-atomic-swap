import * as React from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';

export default function Withdraw(props) {
  console.log(props.swapDetails);
  console.log(props.ongoingSwap);
  
  return (
    <div>
      <React.Fragment>
        <Typography variant="h6" color="inherit" noWrap sx={{ mx: 1 }}>
          Withdraw
        </Typography>
        <Box sx={{ display: 'flex', justifyContent: 'center' }}>
          <Button
            variant="contained"
            onClick={props.onCompletion}
            sx={{  mt: 3, ml: 1, borderRadius: 8, textTransform: 'none' }} 
          > Withdraw
          </Button>
        </Box>
      </React.Fragment>
    </div>
  );
}
