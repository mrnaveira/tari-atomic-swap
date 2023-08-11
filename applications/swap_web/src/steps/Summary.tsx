import * as React from 'react';
import Typography from '@mui/material/Typography';
import Button from '@mui/material/Button';
import Box from '@mui/material/Box';

export default function LockFunds(props) {
  console.log(props.swapDetails);

  return (
    <div>
      <React.Fragment>
        <Typography variant="h6" color="inherit" noWrap sx={{ mx: 1 }}>
          Summary
        </Typography>
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
